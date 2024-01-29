# DHT11/DHT22 sensor driver

[![crates.io](https://img.shields.io/crates/v/dht-sensor)](https://crates.io/crates/dht-sensor)
[![Docs](https://docs.rs/dht-sensor/badge.svg)](https://docs.rs/dht-sensor)

This library provides a platform-agnostic driver for the [DHT11 and DHT22](https://learn.adafruit.com/dht/overview) sensors.

Use one of two functions `dht11::Reading::read` and `dht22::Reading::read` to get a reading.

## Usage

The only prerequisites are an embedded-hal implementation that provides:

- `Delay`-implementing type, for example Cortex-M microcontrollers typically use the `SysTick`.
- `InputOutputPin`-implementing type, for example an `Output<OpenDrain>` from `stm32f0xx_hal`.
  - Note that you'll almost certainly need to configure your pin as open drain. See [#23](https://github.com/michaelbeaumont/dht-sensor/issues/23) for some discussion.

See the [stm32f042 example](examples/stm32f042.rs) for a commented example of
how to use the library.

### Release mode may be required

Compiling in debug mode may disturb the timing-sensitive parts of this crate and ultimately lead to failure.
In this case, you will likely receive a `Timeout` error. Try compiling with `--release` instead.

### Tests

To run the tests, use something like `cargo test --lib --target x86_64-unknown-linux-gnu`.
