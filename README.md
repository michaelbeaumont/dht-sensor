# DHT11/DHT22 sensor driver

[![crates.io](https://img.shields.io/crates/v/dht-sensor)](https://crates.io/crates/dht-sensor)
[![Docs](https://docs.rs/dht-sensor/badge.svg)](https://docs.rs/dht-sensor)

This library provides a platform-agnostic driver for the [DHT11 and DHT22](https://learn.adafruit.com/dht/overview) sensors.

Use one of two functions `dht11::blocking::read` and `dht22::blocking::read` to get a reading.

## Usage

The only prerequisites are an embedded-hal implementation that provides:

- `DelayNs`-implementing type, for example Cortex-M microcontrollers typically use the `SysTick`.
- `InputPin` and `OutputPin`-implementing type, for example an `Output<OpenDrain>` from `stm32f0xx_hal`.

When initializing the pin as an output, the state of the pin might depend on the specific chip
used. Some might pull the pin low by default causing the sensor to be confused when we actually
read it for the first time. The same thing happens when the sensor is polled too quickly in succession.
In both of those cases you will get a `DhtError::Timeout`.

To avoid this, you can pull the pin high when initializing it and polling the sensor with an
interval of at least 500ms (determined experimentally). Some sources state a refresh rate of 1 or even 2 seconds.

## Example

See the following examples for how to use the library.

### Blocking API

- [stm32f051r8](examples/stm32f051r8/src/bin/sync.rs)

### Async API

- [stm32f051r8](examples/stm32f051r8/src/bin/async.rs)
- [stm32f303vc](examples/stm32f303vc/src/bin/async.rs)

### Release mode may be required

Compiling in debug mode may disturb the timing-sensitive parts of this crate and ultimately lead to failure.
In this case, you will likely receive a `Timeout` error. Try compiling with `--release` instead.

### Tests

To run the tests, use something like `cargo test --lib --target x86_64-unknown-linux-gnu`.
