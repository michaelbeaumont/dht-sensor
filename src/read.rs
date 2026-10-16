use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

const TIMEOUT_US: u8 = 100;

#[derive(Debug)]
pub enum DhtError<E> {
    PinError(E),
    /// Carries what was read, to tell a single flipped bit from garbage.
    ChecksumMismatch { data: [u8; 4], checksum: u8 },
    Timeout(Stage),
}

/// The step of the transmission that timed out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Waiting for the sensor to end its ~80µs low "I'm here" pulse.
    AckLow,
    /// Waiting for the sensor to end its ~80µs high pulse before the data.
    /// Also what a sensor that doesn't answer at all gives, since the line
    /// then simply stays high.
    AckHigh,
    /// Waiting for bit `n` (0..40) to start, i.e. its ~50µs low to end.
    BitStart(u8),
    /// Waiting for bit `n` (0..40) to end, i.e. its high part to end.
    BitEnd(u8),
}

impl<E> From<E> for DhtError<E> {
    fn from(error: E) -> DhtError<E> {
        DhtError::PinError(error)
    }
}

fn read_bit<P: InputPin>(
    delay: &mut impl DelayNs,
    pin: &mut P,
    bit: u8,
) -> Result<bool, DhtError<P::Error>> {
    wait_until_timeout(delay, Stage::BitStart(bit), || pin.is_high())?;
    delay.delay_us(35);
    let high = pin.is_high()?;
    wait_until_timeout(delay, Stage::BitEnd(bit), || pin.is_low())?;
    Ok(high)
}

fn read_byte<P: InputPin>(
    delay: &mut impl DelayNs,
    pin: &mut P,
    index: u8,
) -> Result<u8, DhtError<P::Error>> {
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - i);
        if read_bit(delay, pin, index * 8 + i)? {
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

    wait_until_timeout(delay, Stage::AckLow, || pin.is_high())?;
    wait_until_timeout(delay, Stage::AckHigh, || pin.is_low())?;

    let mut data = [0; 4];
    for (index, b) in (0..).zip(data.iter_mut()) {
        *b = read_byte(delay, pin, index)?;
    }
    let checksum = read_byte(delay, pin, 4)?;
    if data.iter().fold(0u8, |sum, v| sum.wrapping_add(*v)) != checksum {
        Err(DhtError::ChecksumMismatch { data, checksum })
    } else {
        Ok(data)
    }
}

/// Wait until the given function returns true or the timeout is reached,
/// in which case `stage` says what we were waiting for.
fn wait_until_timeout<E, F>(
    delay: &mut impl DelayNs,
    stage: Stage,
    mut func: F,
) -> Result<(), DhtError<E>>
where
    F: FnMut() -> Result<bool, E>,
{
    for _ in 0..TIMEOUT_US {
        if func()? {
            return Ok(());
        }
        delay.delay_us(1);
    }
    Err(DhtError::Timeout(stage))
}
