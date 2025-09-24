# Copilot Instructions

## Project Overview
This repository contains two primary components:

1. **`esp/`**: A Rust-based embedded project targeting ESP32 devices. It includes the main application logic in `src/main.rs` and uses the `esp-idf` ecosystem for hardware abstraction and system services.
2. **`mess_lib/`**: A Rust library (`lib.rs`) that provides reusable functionality for the `esp` project and potentially other projects. It is structured as a standalone library crate.

## Architecture and Data Flow
- The `esp` project is the entry point for the embedded application. It interacts with hardware peripherals and system services via the `esp-idf` HAL (Hardware Abstraction Layer).
- The `mess_lib` crate encapsulates shared logic and utilities, promoting modularity and reusability.
- Communication between components is achieved through Rust's module and crate system, ensuring type safety and clear boundaries.

## Developer Workflows

### Building the Project
- To build the `esp` project, use the following command:
  ```sh
  cargo build
  ```
  Ensure that the appropriate toolchain for ESP32 development is installed (e.g., `espup` or `esp-idf` setup).

- To build the `mess_lib` library:
  ```sh
  cargo build --manifest-path=mess_lib/Cargo.toml
  ```

### Running Tests
- Unit tests for `mess_lib` can be executed with:
  ```sh
  cargo test --manifest-path=mess_lib/Cargo.toml
  ```
- Integration tests are located in `mess_lib/tests/` and can be run similarly.

### Debugging
- Use `cargo run` for debugging the `esp` project. Ensure the target device is connected and properly configured.
- For `mess_lib`, standard Rust debugging tools (e.g., `gdb`, `lldb`) can be used.

## Project-Specific Conventions
- Follow the Rust module organization pattern: public APIs are exposed via `lib.rs` in `mess_lib`.
- Use `sdkconfig.defaults` in `esp` to configure ESP32-specific settings.
- Avoid hardcoding values; use constants or configuration files where possible.

## External Dependencies
- The `esp` project relies on the `esp-idf` ecosystem. Ensure the ESP-IDF environment is set up correctly.
- Common crates used include `anyhow`, `serde`, and `embedded-hal`.

## Key Files and Directories
- `esp/src/main.rs`: Entry point for the embedded application.
- `mess_lib/src/lib.rs`: Core library functionality.
- `mess_lib/tests/`: Integration tests for the library.
- `esp/sdkconfig.defaults`: Default configuration for ESP32.

## Examples
- To add a new module to `mess_lib`, create a new file in `src/` and expose it in `lib.rs`:
  ```rust
  pub mod new_module;
  ```
- To use `mess_lib` in `esp`, add it as a dependency in `Cargo.toml`:
  ```toml
  [dependencies]
  mess_lib = { path = "../mess_lib" }
  ```

---

Feel free to update this document as the project evolves.