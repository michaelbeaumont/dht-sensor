# Changelog

## 0.3.0

### Breaking changes

The module structure has changed. The existing `dht*::Reading::read` functions have moved to `dht*::blocking::read`.

### Features

* New `dht*::async` module for async support
* defmt support

### Dependency updates

* embedded-hal v1.0

## 0.2.1

### Fixes

* Fix add with overflow for debug builds (#6)

## 0.2.0

### Features

* Add timeouts in read functions for reliability (#2)
* Use `dyn Delay` instead of generic type

## 0.1.1

### Fixes

* build: Add metadata to `Cargo.toml` to fix https://docs.rs build

## 0.1.0

### Features

* Use one of two functions `dht11::Reading::read` and `dht22::Reading::read` to get a reading
