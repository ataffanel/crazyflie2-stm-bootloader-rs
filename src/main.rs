#![no_main]
#![no_std]

use crazyflie2_stm_bootloader as _; // global logger + panicking-behavior + memory layout

use cortex_m;
use stm32f4xx_hal as hal;

use crate::hal::{prelude::*, stm32, stm32::{interrupt, Interrupt}};
use stm32f4xx_hal::stm32::USART6;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use heapless::{spsc::{Queue, Producer, Consumer}};
use heapless::consts::*;

mod flash;
mod syslink;
mod protocol;

static USART_RX: Mutex<RefCell<Option<hal::serial::Rx<USART6>>>> = Mutex::new(RefCell::new(None));
static mut USART_QUEUE: Queue<u8, U64, usize> = Queue(heapless::i::Queue::new());

#[interrupt]
fn USART6() {
    static mut RX: Option<hal::serial::Rx<USART6>> = None;
    static mut QUEUE: Option<Producer<u8, U64, usize>> = None;

    let rx = RX.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            USART_RX.borrow(cs).replace(None).unwrap()
        })
    });

    let queue = QUEUE.get_or_insert_with(|| {
            unsafe { USART_QUEUE.split().0 }
    });

    if let Ok(data) = rx.read() {
        if queue.ready() {
            queue.enqueue(data).unwrap();
        }
    }
}

struct QueuedRead<'a>  {
    rx_queue: Consumer<'a, u8, U64, usize>,
}

impl QueuedRead<'_> {
    fn new() -> Self {
        let consumer = unsafe { USART_QUEUE.split().1 };

        QueuedRead {
            rx_queue: consumer
        }
    }
}

impl embedded_hal::serial::Read<u8> for QueuedRead<'_> {
    type Error = ();

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if self.rx_queue.ready() {
            Ok(self.rx_queue.dequeue().unwrap())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Hello, {:?} world!", 42);

    if let (Some(dp), Some(cp)) = (stm32::Peripherals::take(), cortex_m::peripheral::Peripherals::take()) {
        // Setup LED
        let gpiod = dp.GPIOD.split();
        let mut blue_led = gpiod.pd2.into_push_pull_output();

        blue_led.set_low().unwrap();

        // Serup boot control pin/nrf flow control pin
        let gpioa = dp.GPIOA.split();
        let nrf_flow_control = gpioa.pa4.into_pull_down_input();

        // Setup system clock
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        // Setup a 2Hz asynchronous timer using systick (to blink the LED)
        let mut timer = hal::timer::Timer::tim1(dp.TIM1, 2.hz(), clocks);
        
        // Check boot pin and check if the firmware is not in an erased sector
        if nrf_flow_control.is_low().unwrap() && get_firmware_stack_pointer() != 0xffff_ffff {
            boot_firmware(cp.SCB);
        } else {

            // Setup UART to nRF51
            let gpioc = dp.GPIOC.split();
            
            let tx = gpioc.pc6.into_alternate_af8();
            let rx = gpioc.pc7.into_alternate_af8();

            let mut serial = hal::serial::Serial::usart6(
                dp.USART6, (tx, rx), 
                hal::serial::config::Config::default().baudrate(1000000.bps()),
                clocks).unwrap();
            
            // Enable RX Not Empty interrupt
            serial.listen(hal::serial::Event::Rxne);

            let (tx, rx) = serial.split();

            // Pass RX side to the interrupt via a static variable
            cortex_m::interrupt::free(|cs| *USART_RX.borrow(cs).borrow_mut() = Some(rx));

            // Create queued-read object that reads from the global RX queue
            let rx = QueuedRead::new();

            // Enable USART6 interrupt
            unsafe { cortex_m::peripheral::NVIC::unmask(Interrupt::USART6); }
            
            // Create syslink handler
            let mut syslink = syslink::Syslink::new(rx, tx);

            // Flash access
            let mut flash = flash::Flash::new(dp.FLASH);

            // Buffer to hold data to be flashed
            let mut buffer: [[u8; 1024]; 10] = [[0u8; 1024]; 10];
            
            // Main loop
            loop {
                if let Ok(mut packet) = syslink.receive() {
                    // defmt::info!("Received packet of type {:u8} and size {:?}", packet.packet_type, packet.length);
                    if protocol::handle_packet(&mut packet, &mut buffer, &mut flash) {
                        packet.set_checksum();
                        nb::block!(syslink.send(&packet)).unwrap();
                    }
                    
                }

                if let Ok(_) = timer.wait() {
                    blue_led.toggle().unwrap();
                }
            }    
        }

        
    }

    crazyflie2_stm_bootloader::exit()
}

fn boot_firmware(scb: cortex_m::peripheral::SCB) -> ! {
    unsafe {
        let firmware_entry = (0x08004000 + 4) as *const fn() -> !;
        let firmware_stack = 0x08004000 as *const u32;
        
        scb.vtor.write(0x08004000);
        cortex_m::register::msp::write(*firmware_stack);
        (*firmware_entry)();
    }
}

fn get_firmware_stack_pointer() -> u32{
    unsafe {
        let firmware_stack = 0x08004000 as *const u32;
        *firmware_stack
    }
}
