#![no_std]
#![no_main]

use cortex_m_semihosting::hprintln;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Flex, Pull, Speed};
use embassy_time::Delay;
use embedded_hal_async::delay::DelayNs as _;
use panic_halt as _;

use dht_sensor::*;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum DeviceSelect {
    Dht11,
    Dht22,
}

const DEV_SEL: DeviceSelect = DeviceSelect::Dht22;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    // This is used by `dht-sensor` to wait for signals
    let mut delay = Delay;

    let mut one_wire_pin = Flex::new(p.PA1);
    one_wire_pin.set_as_input_output_pull(Speed::VeryHigh, Pull::Up);

    // Pulling the pin high to avoid confusing the sensor when initializing
    one_wire_pin.set_high();

    // The DHT11 datasheet suggests 1 second
    hprintln!("Waiting on the sensor...");
    delay.delay_ms(1000).await;

    loop {
        if DEV_SEL == DeviceSelect::Dht22 {
            match dht22::r#async::read(&mut delay, &mut one_wire_pin).await {
                Ok(dht22::Reading {
                    temperature,
                    relative_humidity,
                }) => hprintln!("{}°, {}% RH", temperature, relative_humidity),
                Err(e) => hprintln!("Error {:?}", e),
            }
            // Delay of at least 500ms before polling the sensor again, 1 second or more advised
            delay.delay_ms(500).await;
        } else {
            match dht11::r#async::read(&mut delay, &mut one_wire_pin).await {
                Ok(dht11::Reading {
                    temperature,
                    relative_humidity,
                }) => hprintln!("{}°, {}% RH", temperature, relative_humidity),
                Err(e) => hprintln!("Error {:?}", e),
            }
            // Delay of at least 500ms before polling the sensor again, 1 second or more advised
            delay.delay_ms(1000).await;
        }
    }
}
