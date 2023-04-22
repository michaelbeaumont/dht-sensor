use core::future::Future;
use embassy_futures::select::{select, Either};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_async::{delay::DelayUs, digital::Wait};

use crate::error::*;

const TIMEOUT: u32 = 1000;

async fn read_bit(delay: &mut impl DelayUs, pin: &mut impl Wait) -> Result<bool, DhtError> {
    await_with_timeout(delay, pin.wait_for_rising_edge()).await?;
    Ok(matches!(
        select(pin.wait_for_low(), delay.delay_us(28)).await,
        Either::Second(_)
    ))
}

async fn read_byte(delay: &mut impl DelayUs, pin: &mut impl Wait) -> Result<u8, DhtError> {
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - (i % 8));
        if read_bit(delay, pin).await? {
            byte |= bit_mask;
        }
    }
    Ok(byte)
}

pub async fn read_raw<D, T>(delay: &mut D, pin: &mut T) -> Result<[u8; 4], DhtError>
where
    D: DelayUs,
    T: Wait + OutputPin,
{
    let mut data = [0; 4];
    pin.set_low().ok();
    delay.delay_ms(18).await;
    pin.set_high().ok();

    await_with_timeout(delay, pin.wait_for_low()).await?;
    await_with_timeout(delay, pin.wait_for_high()).await?;
    await_with_timeout(delay, pin.wait_for_low()).await?;

    for b in data.iter_mut() {
        *b = read_byte(delay, pin).await?;
    }
    let checksum = read_byte(delay, pin).await?;
    if data.iter().fold(0u8, |sum, v| sum.wrapping_add(*v)) != checksum {
        Err(DhtError::ChecksumMismatch)
    } else {
        Ok(data)
    }
}

async fn await_with_timeout(delay: &mut impl DelayUs, future: impl Future) -> Result<(), DhtError> {
    match select(future, delay.delay_ms(TIMEOUT)).await {
        Either::First(_) => Ok(()),
        Either::Second(_) => Err(DhtError::Timeout),
    }
}
