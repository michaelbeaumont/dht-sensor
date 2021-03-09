use core::pin::Pin;
use embassy::time::Instant;
use embassy::traits::{delay::Delay, gpio::*};
use embedded_hal::digital::v2::InputPin;

use crate::error::*;
use crate::pin::*;

async fn read_bit<T, E>(mut pin: Pin<&mut T>) -> Result<bool, DhtError<E>>
where
    T: WaitForHigh + WaitForLow + InputPin<Error = E>,
{
    pin.as_mut().wait_for_high().await;
    let start = Instant::now();
    pin.as_mut().wait_for_low().await;
    // This is fragile because of TICKS_PER_SECOND in embassy but appears to work 99% of the time
    Ok(start.elapsed().as_micros() > 40)
}

async fn read_byte<T, E>(mut pin: Pin<&mut T>) -> Result<u8, DhtError<E>>
where
    T: WaitForHigh + WaitForLow + InputPin<Error = E>,
{
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - (i % 8));
        if read_bit(pin.as_mut()).await? {
            byte |= bit_mask;
        }
    }
    Ok(byte)
}

pub async fn read_raw<E, D, T>(
    mut delay: Pin<&mut D>,
    mut pin: Pin<&mut T>,
) -> Result<[u8; 4], DhtError<E>>
where
    D: Delay,
    T: Unpin + WaitForHigh + WaitForLow + InputOutputPin<E>,
{
    let mut data = [0; 4];
    pin.set_low().ok();
    delay.as_mut().delay_ms(18).await;
    pin.set_high().ok();

    pin.as_mut().wait_for_low().await;
    pin.as_mut().wait_for_high().await;
    pin.as_mut().wait_for_low().await;

    for b in data.iter_mut() {
        *b = read_byte(pin.as_mut()).await?;
    }
    let checksum = read_byte(pin).await?;
    if data.iter().fold(0u8, |sum, v| sum.wrapping_add(*v)) != checksum {
        Err(DhtError::ChecksumMismatch)
    } else {
        Ok(data)
    }
}
