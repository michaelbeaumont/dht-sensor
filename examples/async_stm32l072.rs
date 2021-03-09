#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

use crate::hal::{exti::Exti, pac, prelude::*, rcc, syscfg};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embassy::{
    executor::{task, Executor},
    time,
    traits::delay::Delay,
    util::Forever,
};
use embassy_stm32l0::{exti, interrupt, rtc};
use futures::pin_mut;
use panic_halt as _;
use stm32l0xx_hal as hal;

use dht_sensor::*;

static ALARM: Forever<rtc::Alarm<pac::TIM2>> = Forever::new();
static EXECUTOR: Forever<Executor> = Forever::new();
static EXTI: Forever<exti::ExtiManager> = Forever::new();
static RTC: Forever<rtc::RTC<pac::TIM2>> = Forever::new();

#[task]
async fn run(mut rcc: rcc::Rcc, gpioa: pac::GPIOA, exti: pac::EXTI, syscfg: pac::SYSCFG) {
    let gpioa = gpioa.split(&mut rcc);

    let mut button = gpioa.pa4.into_open_drain_output();
    button.set_high().ok();

    let exti = Exti::new(exti);
    let syscfg = syscfg::SYSCFG::new(syscfg, &mut rcc);
    let exti = EXTI.put(exti::ExtiManager::new(exti, syscfg));

    let delay = time::Delay::new();
    pin_mut!(delay);

    let pin = exti.new_pin(button, interrupt::take!(EXTI4_15));
    pin_mut!(pin);

    loop {
        delay.as_mut().delay_ms(1000).await;
        match dht11::read(delay.as_mut(), pin.as_mut()).await {
            Ok(dht11::Reading {
                temperature,
                relative_humidity,
            }) => hprintln!("{}°, {}% RH", temperature, relative_humidity).unwrap(),
            Err(e) => hprintln!("Error {:?}", e).unwrap(),
        };
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());

    let rtc = RTC.put(rtc::RTC::new(dp.TIM2, interrupt::take!(TIM2), rcc.clocks));
    rtc.start();

    let alarm = ALARM.put(rtc.alarm1());
    unsafe { embassy::time::set_clock(rtc) };

    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);

    let gpioa = dp.GPIOA;
    let exti = dp.EXTI;
    let sc = dp.SYSCFG;

    executor.run(|spawner| {
        spawner.spawn(run(rcc, gpioa, exti, sc)).unwrap();
    });
}
