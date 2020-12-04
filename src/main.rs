#![no_main]
#![no_std]

use crazyflie2_stm_bootloader as _; // global logger + panicking-behavior + memory layout

use cortex_m;
use stm32f4xx_hal as hal;

use crate::hal::{prelude::*, stm32};

mod flash;
mod syslink;

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

            let serial = hal::serial::Serial::usart6(
                dp.USART6, (tx, rx), 
                hal::serial::config::Config::default().baudrate(1000000.bps()),
                clocks).unwrap();

            let (tx, rx) = serial.split();
            
            // Create syslink handler
            let mut syslink = syslink::Syslink::new(rx, tx);
            
            // Main loop
            loop {
                if let Ok(packet) = syslink.receive() {
                    defmt::info!("Received packet of type {:u8} and size {:?}", packet.packet_type, packet.length);
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
