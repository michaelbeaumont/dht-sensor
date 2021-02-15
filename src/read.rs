use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};

#[derive(Debug)]
pub enum DhtError<E> {
    PinError(E),
    ChecksumMismatch,
    Timeout,
}

impl<E> From<E> for DhtError<E> {
    fn from(error: E) -> DhtError<E> {
        DhtError::PinError(error)
    }
}

pub trait Delay: DelayUs<u8> + DelayMs<u8> {}
impl<T> Delay for T where T: DelayMs<u8> + DelayUs<u8> {}

pub trait InputOutputPin<E>: InputPin<Error = E> + OutputPin<Error = E> {}
impl<T, E> InputOutputPin<E> for T where T: InputPin<Error = E> + OutputPin<Error = E> {}

fn read_bit<D, E>(delay: &mut D, pin: &impl InputPin<Error = E>) -> Result<bool, DhtError<E>>
where
    D: DelayUs<u8>,
{
    while_with_timeout(delay, || pin.is_low(), 100)?;
    delay.delay_us(35u8);
    let high = pin.is_high()?;
    while_with_timeout(delay, || pin.is_high(), 100)?;
    Ok(high)
}

fn read_byte<D, E>(delay: &mut D, pin: &impl InputPin<Error = E>) -> Result<u8, DhtError<E>>
where
    D: DelayUs<u8>,
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

pub fn read_raw<P, E, D>(delay: &mut D, pin: &mut P) -> Result<[u8; 4], DhtError<E>>
where
    P: InputOutputPin<E>,
    D: Delay,
{
    pin.set_low().ok();
    delay.delay_ms(18_u8);
    pin.set_high().ok();
    delay.delay_us(48_u8);

    while_with_timeout(delay, || pin.is_low(), 100)?;
    while_with_timeout(delay, || pin.is_high(), 100)?;

    let mut data = [0; 4];
    for b in data.iter_mut() {
        *b = read_byte(delay, pin)?;
    }
    let checksum = read_byte(delay, pin)?;
    if data.iter().sum::<u8>() != checksum {
        Err(DhtError::ChecksumMismatch)
    } else {
        Ok(data)
    }
}

/// Loop while the given function returns true or the timeout is reached.
fn while_with_timeout<E, D, F>(delay: &mut D, func: F, timeout_us: u16) -> Result<(), DhtError<E>> 
where
    D: DelayUs<u8>,
    F: Fn() -> Result<bool, E>
{
    let mut count = 0;

    while func()? {
        delay.delay_us(10_u8);
        count += 1;
        if count >= timeout_us {
            return Err(DhtError::Timeout);
        }
    }
    Ok(())
}