#![no_std]

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub struct Reading {
    pub temperature: i8,
    pub humidity: u8,
}

#[derive(Debug)]
pub enum DhtError<E> {
    PinError(E),
    ChecksumMismatch,
}

impl<E> From<E> for DhtError<E> {
    fn from(error: E) -> DhtError<E> {
        DhtError::PinError(error)
    }
}

fn read_bit<D, E>(delay: &mut D, pin: &impl InputPin<Error = E>) -> Result<bool, E>
where
    D: DelayUs<u8>,
    E: core::fmt::Debug,
{
    while pin.is_low()? {}
    delay.delay_us(35u8);
    let high = pin.is_high()?;
    while pin.is_high()? {}
    Ok(high)
}

fn read_byte<D, E>(delay: &mut D, pin: &impl InputPin<Error = E>) -> Result<u8, E>
where
    D: DelayUs<u8>,
    E: core::fmt::Debug,
{
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - (i % 8));
        if read_bit(delay, pin)? {
            byte |= bit_mask;
        }
    }
    Ok(byte)
}

pub fn read11<P, E, D>(delay: &mut D, pin: &mut P) -> Result<Reading, DhtError<E>>
where
    P: InputPin<Error = E> + OutputPin<Error = E>,
    E: core::fmt::Debug,
    D: DelayMs<u8> + DelayUs<u8>,
{
    pin.set_low().ok();
    delay.delay_ms(18_u8);
    pin.set_high().ok();
    delay.delay_us(48_u8);
    while pin.is_low()? {}
    while pin.is_high()? {}
    let humidity = read_byte(delay, pin)?;
    let _ = read_byte(delay, pin)?;
    let temp_signed = read_byte(delay, pin)?;
    let temperature = {
        let temp_sign = if temp_signed & 0x80 != 0 { -1 } else { 1 };
        let temp_magnitude = temp_signed & 0x7F;
        temp_sign * temp_magnitude as i8
    };
    let _ = read_byte(delay, pin)?;
    let checksum = read_byte(delay, pin)?;
    if humidity + temperature != checksum {
        Err(DhtError::ChecksumMismatch)
    } else {
        Ok(Reading {
            temperature,
            humidity,
        })
    }
}
