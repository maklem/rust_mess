# Measurement Board using Embedded Rust

## Abstract

Some time ago I assembled a measurement board for some environmental data (brightness, temperature, air quality) using Arduino on ESP32.
I now want to learn Rust, and I took that project as target state, with addidion of professional development strategies (tests, CI).

current state: main framework is there, but no functionality yet.

## What works, and what does not (yet)

* ✅ Platform Independent Support Library
  * ✅ use from ESP32 platform
  * ✅ compile and test in CI
* 🟡 ESP32 Board Implementation
  * ✅ build, flash, run Firmware for ESP32
  * ✅ [debugging firmware on hardware (using JTAG)](docs/hardware-debugging.md)
  * ✅ unit tests on hardware (using embedded-test)
  * ❌ no hardware debugging in embedded tests
    cargo-test uses probe-rs, but not in a way suitable for vscode; atleast as far as I have figured it out
  * ❓ Networking
  * ❓ MQTT
  * ✅ build in CI
  * ❌ flash / test in CI -- hardware can not be attached to public Github-Actions runners
    ➡️ might work using a custom runner (I don't have enough spare hardware though)
    ➡️ core-lib is tested in CI (see above)

## References

* Rust on ESP Book - Espressif
  https://docs.espressif.com/projects/rust/book/preface.html
