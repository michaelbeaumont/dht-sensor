#![no_std]
#![no_main]

use crate::hal::{delay, gpio, prelude::*, stm32};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_halt as _;
use stm32f0xx_hal as hal;

use dht_sensor::*;

#[entry]
fn main() -> ! {
    let mut p = stm32::Peripherals::take().unwrap();
    let cp = stm32::CorePeripherals::take().unwrap();
    let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

    // This is used by `dht-sensor` to wait for signals
    let mut delay = delay::Delay::new(cp.SYST, &rcc);

    // This could be any `gpio` port
    let gpio::gpioa::Parts { pa1, .. } = p.GPIOA.split(&mut rcc);

    // An `Output<OpenDrain>` is both `InputPin` and `OutputPin`
    let mut pa1 = cortex_m::interrupt::free(|cs| pa1.into_open_drain_output(cs));

    // Pulling the pin high to avoid confusing the sensor when initializing
    pa1.set_high().ok();

    // The DHT11 datasheet suggests 1 second
    hprintln!("Waiting on the sensor...");
    delay.delay_ms(1000_u16);

    loop {
        match dht11::read(&mut delay, &mut pa1) {
            Ok(dht11::Reading {
                temperature,
                relative_humidity,
            }) => hprintln!("{}°, {}% RH", temperature, relative_humidity),
            Err(e) => hprintln!("Error {:?}", e),
        }

        // Delay of at least 500ms before polling the sensor again, 1 second or more advised
        delay.delay_ms(500_u16);
    }
}
