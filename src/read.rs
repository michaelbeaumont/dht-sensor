use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};


use crate::error::*;

pub trait Delay: DelayUs<u8> + DelayMs<u8> {}
impl<T> Delay for T where T: DelayMs<u8> + DelayUs<u8> {}

fn read_bit(delay: &mut dyn Delay, pin: &impl InputPin) -> Result<bool, DhtError> {
    wait_until_timeout(delay, || pin.is_high(), 100)?;
    delay.delay_us(35u8);
    let high = pin.is_high().map_err(|_| DhtError::PinError)?;
    wait_until_timeout(delay, || pin.is_low(), 100)?;
    Ok(high)
}

fn read_byte(delay: &mut dyn Delay, pin: &impl InputPin) -> Result<u8, DhtError> {
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - (i % 8));
        if read_bit(delay, pin)? {
            byte |= bit_mask;
        }
    }
    Ok(byte)
}

pub fn read_raw<P>(delay: &mut dyn Delay, pin: &mut P) -> Result<[u8; 4], DhtError>
where
    P: InputPin + OutputPin,
{
    pin.set_low().ok();
    delay.delay_ms(18_u8);
    pin.set_high().ok();
    delay.delay_us(48_u8);

    wait_until_timeout(delay, || pin.is_high(), 100)?;
    wait_until_timeout(delay, || pin.is_low(), 100)?;

    let mut data = [0; 4];
    for b in data.iter_mut() {
        *b = read_byte(delay, pin)?;
    }
    let checksum = read_byte(delay, pin)?;
    if data.iter().fold(0u8, |sum, v| sum.wrapping_add(*v)) != checksum {
        Err(DhtError::ChecksumMismatch)
    } else {
        Ok(data)
    }
}

/// Wait until the given function returns true or the timeout is reached.
fn wait_until_timeout<F, E>(delay: &mut dyn Delay, func: F, timeout_us: u8) -> Result<(), DhtError>
where
    F: Fn() -> Result<bool, E>,
{
    for _ in 0..timeout_us {
        if func().map_err(|_| DhtError::PinError)? {
            return Ok(());
        }
        delay.delay_us(1_u8);
    }
    Err(DhtError::Timeout)
}
