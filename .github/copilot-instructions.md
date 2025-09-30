# Copilot Instructions

## Project Overview
This repository contains two primary components:

1. **`esp/`**: A Rust-based embedded project targeting ESP32 devices. It is configured with Rust 1.86 and includes:
   - Main application logic in `src/main.rs`
   - Hardware testing support in `tests/hello_test.rs`
   - ESP-IDF ecosystem integration for hardware abstraction
   - Configured for no-std embedded development

2. **`mess_lib/`**: A Rust 2024 edition library crate that provides:
   - Core functionality in `src/lib.rs`
   - Modular test infrastructure in `tests/hello_test.rs`
   - Clean separation of common code for reuse

## Architecture and Data Flow
- The `esp` project serves as the embedded application entry point with:
  - Direct ESP32 hardware interactions
  - Integration with ESP-IDF HAL for hardware abstraction
  - No-std compatibility for embedded deployment
- The `mess_lib` crate provides:
  - Reusable functionality for the ESP32 project
  - Clear API boundaries through the Rust module system
  - Modern Rust 2024 features for improved development
- Inter-component communication follows Rust's strict type system and module boundaries

## Developer Workflows

### Building the Project
- To build the `esp` project:
  ```sh
  cargo build
  ```
  The project requires:
  - ESP32 development toolchain
  - WiFi credentials (set via `WIFI_SSID` and `WIFI_PASS` environment variables)
  - Rust 1.86 or compatible version

- To build the `mess_lib` library:
  ```sh
  cargo build --manifest-path=mess_lib/Cargo.toml
  ```

### Running Tests
- Hardware tests for `esp` are in `tests/hello_test.rs` (non-standard harness)
- Unit tests for `mess_lib` can be executed with:
  ```sh
  cargo test --manifest-path=mess_lib/Cargo.toml
  ```
- Integration tests are in both `esp/tests/` and `mess_lib/tests/`

### Debugging
- Hardware debugging guidance is provided in `docs/hardware-debugging.md`
- Visual wiring instructions are available in `docs/IMG_wiring.jpg`
- For `mess_lib`, standard Rust debugging tools (e.g., `gdb`, `lldb`) can be used

## Project-Specific Conventions
- No-std environment for the ESP32 project
- Custom test harness for hardware tests
- Clean separation between embedded and library code
- Modern Rust features in `mess_lib` (2024 edition)

## External Dependencies
Major dependencies include:
- Embassy HAL for hardware abstraction
- ESP-IDF integration
- Embedded test framework
- Core async/embedded-io libraries

## Key Files and Directories
- `esp/src/main.rs`: Embedded application entry point
- `esp/tests/hello_test.rs`: Hardware test implementation
- `mess_lib/src/lib.rs`: Core library functionality
- `mess_lib/tests/hello_test.rs`: Library test suite
- `docs/`: Hardware documentation and diagrams

## Examples
- To add embedded functionality, extend `esp/src/main.rs`
- To add shared code, create modules in `mess_lib/src/`
- For hardware tests, follow patterns in `esp/tests/hello_test.rs`
- Library tests should be added to `mess_lib/tests/`

---

Feel free to update this document as the project evolves.