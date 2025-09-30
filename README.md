# Measurement Board using Embedded Rust

## Abstract

Some time ago I assembled a measurement board for some environmental data (brightness, temperature, air quality) using Arduino on ESP32.
I now want to learn Rust, and I took that project as target state, with addidion of common development strategies (tests, CI).

current state: main framework is there, but no functionality yet.

## What works, and what does not (yet)

* ✅ Platform Independent Support Library
  * ✅ use from ESP32 platform
  * ✅ compile and test in CI
* 🟡 ESP32 Board Implementation
  * ✅ build, flash, run Firmware for ESP32
  * ✅ debugging firmware on hardware (using JTAG)
  * ✅ unit tests on hardware (using embedded-test)
  * ❌ no hardware debugging in embedded tests
  * ❓ Networking
  * ❓ MQTT
