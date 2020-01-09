#![no_std]

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};

mod read;
pub use read::DhtError;

fn convert_signed(signed: u8) -> (bool, u8) {
    let sign = signed & 0x80 != 0;
    let magnitude = signed & 0x7F;
    (sign, magnitude)
}

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
            let (signed, magnitude) = convert_signed(temp_signed);
            let temp_sign = if signed { -1 } else { 1 };
            temp_sign * magnitude as i8
        };
        Ok(Reading {
            temperature: temp,
            relative_humidity: rh,
        })
    }
}

pub mod dht22 {
    use super::*;

    pub struct Reading {
        pub temperature: f32,
        pub relative_humidity: f32,
    }

    pub fn read<P, E, D>(delay: &mut D, pin: &mut P) -> Result<Reading, read::DhtError<E>>
    where
        P: InputPin<Error = E> + OutputPin<Error = E>,
        E: core::fmt::Debug,
        D: DelayMs<u8> + DelayUs<u8>,
    {
        let [rh_h, rh_l, temp_h_signed, temp_l] = read::read_raw(delay, pin)?;
        let rh = ((rh_h as u16) << 8 | (rh_l as u16)) as f32 / 10.0;
        let temp = {
            let (signed, magnitude) = convert_signed(temp_h_signed);
            let temp_sign = if signed { -1.0 } else { 1.0 };
            let temp_magnitude = ((magnitude as u16) << 8) | temp_l as u16;
            temp_sign * temp_magnitude as f32 / 10.0
        };
        Ok(Reading {
            temperature: temp,
            relative_humidity: rh,
        })
    }
}
