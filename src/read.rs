use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

const TIMEOUT_US: u8 = 100;

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

fn read_bit<P: InputPin>(
    delay: &mut impl DelayNs,
    pin: &mut P,
) -> Result<bool, DhtError<P::Error>> {
    wait_until_timeout(delay, || pin.is_high())?;
    delay.delay_us(35);
    let high = pin.is_high()?;
    wait_until_timeout(delay, || pin.is_low())?;
    Ok(high)
}

fn read_byte<P: InputPin>(delay: &mut impl DelayNs, pin: &mut P) -> Result<u8, DhtError<P::Error>> {
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - i);
        if read_bit(delay, pin)? {
            byte |= bit_mask;
        }
    }
    Ok(byte)
}

pub fn read_raw<P: OutputPin + InputPin>(
    delay: &mut impl DelayNs,
    pin: &mut P,
) -> Result<[u8; 4], DhtError<P::Error>> {
    pin.set_high().ok();
    delay.delay_us(48);

    wait_until_timeout(delay, || pin.is_high())?;
    wait_until_timeout(delay, || pin.is_low())?;

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
fn wait_until_timeout<E, F>(delay: &mut impl DelayNs, mut func: F) -> Result<(), DhtError<E>>
where
    F: FnMut() -> Result<bool, E>,
{
    for _ in 0..TIMEOUT_US {
        if func()? {
            return Ok(());
        }
        delay.delay_us(1);
    }
    Err(DhtError::Timeout)
}
