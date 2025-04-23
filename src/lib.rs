//! # DHT11/DHT22 sensor driver
//!
//! This library provides a platform-agnostic driver for the [DHT11 and DHT22](https://learn.adafruit.com/dht/overview) sensors.
//!
//! Use one of two functions [`dht11::blocking::read`] and [`dht22::blocking::read`] to get a reading.
//!
//! ## Usage
//!
//! The only prerequisites are an embedded-hal implementation that provides:
//!
//! - [`DelayNs`]-implementing type, for example Cortex-M microcontrollers typically use the `SysTick`.
//! - [`InputPin`] and [`OutputPin`]-implementing type, for example an `Output<OpenDrain>` from `stm32f0xx_hal`.
//!
//! When initializing the pin as an output, the state of the pin might depend on the specific chip
//! used. Some might pull the pin low by default causing the sensor to be confused when we actually
//! read it for the first time. The same thing happens when the sensor is polled too quickly in succession.
//! In both of those cases you will get a `DhtError::Timeout`.
//!
//! To avoid this, you can pull the pin high when initializing it and polling the sensor with an
//! interval of at least 500ms (determined experimentally). Some sources state a refresh rate of 1 or even 2 seconds.
//!
//! ## Example
//!
//! See the following examples for how to use the library.
//!
//! ### Blocking API
//!
//! - [stm32f051r8](https://github.com/michaelbeaumont/dht-sensor/blob/main/examples/stm32f051r8/src/bin/sync.rs)
//!
//! ### Async API
//!
//! - [stm32f051r8](https://github.com/michaelbeaumont/dht-sensor/blob/main/examples/stm32f051r8/src/bin/async.rs)
//! - [stm32f303vc](https://github.com/michaelbeaumont/dht-sensor/blob/main/examples/stm32f303vc/src/bin/async.rs)
#![cfg_attr(not(test), no_std)]

mod read;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
pub use read::DhtError;

pub mod dht11 {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Reading {
        pub temperature: i8,
        pub relative_humidity: u8,
    }

    pub mod blocking {
        use super::DelayNs;
        use super::{raw_to_reading, InputPin, OutputPin, Reading};

        pub fn read<P: OutputPin + InputPin>(
            delay: &mut impl DelayNs,
            pin: &mut P,
        ) -> Result<Reading, super::read::DhtError<P::Error>> {
            pin.set_low()?;
            delay.delay_ms(18);
            super::read::read_raw(delay, pin).map(raw_to_reading)
        }
    }

    #[cfg(feature = "async")]
    pub mod r#async {
        use super::DelayNs;
        use super::{raw_to_reading, InputPin, OutputPin, Reading};
        use embedded_hal_async::delay::DelayNs as AsyncDelayNs;

        /// Only the initial 18ms delay is performed asynchronously.
        ///
        /// The byte and bit read phase is performed with blocking delays.
        pub async fn read<P: OutputPin + InputPin>(
            delay: &mut (impl AsyncDelayNs + DelayNs),
            pin: &mut P,
        ) -> Result<Reading, crate::read::DhtError<P::Error>> {
            pin.set_low()?;
            embedded_hal_async::delay::DelayNs::delay_ms(delay, 18).await;
            crate::read::read_raw(delay, pin).map(raw_to_reading)
        }
    }

    fn raw_to_reading(bytes: [u8; 4]) -> Reading {
        let [relative_humidity, _, temp_signed, _] = bytes;
        let temperature = {
            let (signed, magnitude) = convert_signed(temp_signed);
            let temp_sign = if signed { -1 } else { 1 };
            temp_sign * magnitude as i8
        };
        Reading {
            temperature,
            relative_humidity,
        }
    }

    #[test]
    fn test_raw_to_reading() {
        assert_eq!(
            raw_to_reading([0x32, 0, 0x1B, 0]),
            Reading {
                temperature: 27,
                relative_humidity: 50
            }
        );
        assert_eq!(
            raw_to_reading([0x80, 0, 0x83, 0]),
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
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Reading {
        pub temperature: f32,
        pub relative_humidity: f32,
    }

    pub mod blocking {
        use super::DelayNs;
        use super::{raw_to_reading, InputPin, OutputPin, Reading};
        pub fn read<P: OutputPin + InputPin>(
            delay: &mut impl DelayNs,
            pin: &mut P,
        ) -> Result<Reading, super::read::DhtError<P::Error>> {
            pin.set_low()?;
            delay.delay_ms(1);
            super::read::read_raw(delay, pin).map(raw_to_reading)
        }
    }

    #[cfg(feature = "async")]
    pub mod r#async {
        use super::DelayNs;
        use super::{raw_to_reading, InputPin, OutputPin, Reading};
        use embedded_hal_async::delay::DelayNs as AsyncDelayNs;

        /// Only the initial 1 ms delay is performed asynchronously.
        ///
        /// The byte and bit read phase is performed with blocking delays.
        pub async fn read<P: OutputPin + InputPin>(
            delay: &mut (impl AsyncDelayNs + DelayNs),
            pin: &mut P,
        ) -> Result<Reading, crate::read::DhtError<P::Error>> {
            pin.set_low()?;
            embedded_hal_async::delay::DelayNs::delay_ms(delay, 1).await;
            crate::read::read_raw(delay, pin).map(raw_to_reading)
        }
    }

    fn raw_to_reading(bytes: [u8; 4]) -> Reading {
        let [rh_h, rh_l, temp_h_signed, temp_l] = bytes;
        let relative_humidity = ((rh_h as u16) << 8 | (rh_l as u16)) as f32 / 10.0;
        let temperature = {
            let (signed, magnitude) = convert_signed(temp_h_signed);
            let temp_sign = if signed { -1.0 } else { 1.0 };
            let temp_magnitude = ((magnitude as u16) << 8) | temp_l as u16;
            temp_sign * temp_magnitude as f32 / 10.0
        };
        Reading {
            temperature,
            relative_humidity,
        }
    }

    #[test]
    fn test_raw_to_reading() {
        assert_eq!(
            raw_to_reading([0x02, 0x10, 0x01, 0x1B]),
            Reading {
                temperature: 28.3,
                relative_humidity: 52.8
            }
        );
        assert_eq!(
            raw_to_reading([0x02, 0x90, 0x80, 0x1B]),
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
