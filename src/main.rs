#![no_main]
#![no_std]

use crazyflie2_stm_bootloader as _; // global logger + panicking-behavior + memory layout

use cortex_m;
use stm32f4xx_hal as hal;

use crate::hal::{prelude::*, stm32};

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
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

        // Wait a bit and check boot pin
        delay.delay_ms(1u32);
        if nrf_flow_control.is_low().unwrap() {
            boot_firmware(cp.SCB);
        } else {
            loop {
                blue_led.toggle().unwrap();
                delay.delay_ms(500_u32);
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