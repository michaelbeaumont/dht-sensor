#![no_std]

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};

mod read;
pub use read::DhtError;

pub mod dht11 {
    use super::*;

    pub struct Reading {
        pub temperature: i8,
        pub relative_humidity: u8,
    }

    pub fn read<P, E, D>(delay: &mut D, pin: &mut P) -> Result<Reading, read::DhtError<E>>
    where
        P: InputPin<Error = E> + OutputPin<Error = E>,
        E: core::fmt::Debug,
        D: DelayMs<u8> + DelayUs<u8>,
    {
        let [rh, _, temp_signed, _] = read::read_raw(delay, pin)?;
        let temp = {
            let temp_sign = if temp_signed & 0x80 != 0 { -1 } else { 1 };
            let temp_magnitude = temp_signed & 0x7F;
            temp_sign * temp_magnitude as i8
        };
        Ok(Reading {
            temperature: temp,
            relative_humidity: rh,
        })
    }
}
