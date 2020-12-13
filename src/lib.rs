//! # DHT11/DHT22 sensor driver
//!
//! This library provides a platform-agnostic driver for the [DHT11 and DHT22](https://learn.adafruit.com/dht/overview) sensors.
//!
//! Use one of two functions [`dht11::Reading::read`] and [`dht22::Reading::read`] to get a reading.
//!
//! [`dht11::Reading::read`]: dht11/struct.Reading.html#method.read
//! [`dht22::Reading::read`]: dht22/struct.Reading.html#method.read
//!
//! ## Usage
//!
//! The only prerequisites are an embedded-hal implementation that provides:
//!
//! - [`Delay`]-implementing type, for example Cortex-M microcontrollers typically use the `SysTick`.
//! - [`InputOutputPin`]-implementing type, for example an `Output<OpenDrain>` from `stm32f0xx_hal`.
//!
//!
//! ## Example
//!
//! See the [stm32f042 example](https://github.com/michaelbeaumont/dht-sensor/blob/master/examples/stm32f042.rs) for a working example of
//! how to use the library.
//!
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use crate::hal::{delay, gpio, prelude::*, stm32};
//! use cortex_m_rt::entry;
//! use cortex_m_semihosting::hprintln;
//! use panic_halt as _;
//! use stm32f0xx_hal as hal;
//!
//! use dht_sensor::*;
//!
//! #[entry]
//! fn main() -> ! {
//!     let mut p = stm32::Peripherals::take().unwrap();
//!     let cp = stm32::CorePeripherals::take().unwrap();
//!     let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
//!
//!     // This is used by `dht-sensor` to wait for signals
//!     let mut delay = delay::Delay::new(cp.SYST, &rcc);
//!
//!     // This could be any `gpio` port
//!     let gpio::gpioa::Parts { pa1, .. } = p.GPIOA.split(&mut rcc);
//!
//!     // The DHT11 datasheet suggests 1 second
//!     hprintln!("Waiting on the sensor...").unwrap();
//!     delay.delay_ms(1000_u16);
//!
//!     // An `Output<OpenDrain>` is both `InputPin` and `OutputPin`
//!     let mut pa1 = cortex_m::interrupt::free(|cs| pa1.into_open_drain_output(cs));
//!
//!     match dht11::Reading::read(&mut delay, &mut pa1) {
//!         Ok(dht11::Reading {
//!             temperature,
//!             relative_humidity,
//!         }) => hprintln!("{}°, {}% RH", temperature, relative_humidity).unwrap(),
//!         Err(e) => hprintln!("Error {:?}", e).unwrap(),
//!     }
//!     hprintln!("Looping forever now, thanks!").unwrap();
//!
//!     loop {}
//! }
#![cfg_attr(not(test), no_std)]

mod read;
pub use read::{Delay, DhtError, InputOutputPin};

pub trait DhtReading: internal::FromRaw + Sized {
    fn read<P, E>(delay: &mut dyn Delay, pin: &mut P) -> Result<Self, read::DhtError<E>>
    where
        P: InputOutputPin<E>,
    {
        read::read_raw(delay, pin).map(Self::raw_to_reading)
    }
}

mod internal {
    pub trait FromRaw {
        fn raw_to_reading(bytes: [u8; 4]) -> Self;
    }
}

pub mod dht11 {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Reading {
        pub temperature: i8,
        pub relative_humidity: u8,
    }

    impl internal::FromRaw for Reading {
        fn raw_to_reading(bytes: [u8; 4]) -> Reading {
            let [rh, _, temp_signed, _] = bytes;
            let temp = {
                let (signed, magnitude) = convert_signed(temp_signed);
                let temp_sign = if signed { -1 } else { 1 };
                temp_sign * magnitude as i8
            };
            Reading {
                temperature: temp,
                relative_humidity: rh,
            }
        }
    }

    impl DhtReading for Reading {}

    #[test]
    fn test_raw_to_reading() {
        use super::internal::FromRaw;

        assert_eq!(
            Reading::raw_to_reading([0x32, 0, 0x1B, 0]),
            Reading {
                temperature: 27,
                relative_humidity: 50
            }
        );
        assert_eq!(
            Reading::raw_to_reading([0x80, 0, 0x83, 0]),
            Reading {
                temperature: -3,
                relative_humidity: 128
            }
        );
    }
}

pub mod dht22 {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Reading {
        pub temperature: f32,
        pub relative_humidity: f32,
    }

    impl internal::FromRaw for Reading {
        fn raw_to_reading(bytes: [u8; 4]) -> Reading {
            let [rh_h, rh_l, temp_h_signed, temp_l] = bytes;
            let rh = ((rh_h as u16) << 8 | (rh_l as u16)) as f32 / 10.0;
            let temp = {
                let (signed, magnitude) = convert_signed(temp_h_signed);
                let temp_sign = if signed { -1.0 } else { 1.0 };
                let temp_magnitude = ((magnitude as u16) << 8) | temp_l as u16;
                temp_sign * temp_magnitude as f32 / 10.0
            };
            Reading {
                temperature: temp,
                relative_humidity: rh,
            }
        }
    }

    impl DhtReading for Reading {}

    #[test]
    fn test_raw_to_reading() {
        use super::internal::FromRaw;

        assert_eq!(
            Reading::raw_to_reading([0x02, 0x10, 0x01, 0x1B]),
            Reading {
                temperature: 28.3,
                relative_humidity: 52.8
            }
        );
        assert_eq!(
            Reading::raw_to_reading([0x02, 0x90, 0x80, 0x1B]),
            Reading {
                temperature: -2.7,
                relative_humidity: 65.6
            }
        );
    }
}

fn convert_signed(signed: u8) -> (bool, u8) {
    let sign = signed & 0x80 != 0;
    let magnitude = signed & 0x7F;
    (sign, magnitude)
}

#[test]
fn test_convert_signed() {
    assert_eq!(convert_signed(0x13), (false, 0x13));
    assert_eq!(convert_signed(0x93), (true, 0x13));
}
