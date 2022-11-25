# Independent Watchdog (IWDG) Example
This example showcases how the IWDG can be used. The watchdog is set for 500ms and is fed by a software task every 100ms,
if everything goes well. To showcase the working of the watchdog, a user button (connected to `PC13`) can be used to disable
further feeding of the watchdog, leading to it starving and triggering a device reset.

The example logs messages using [`defmt`](https://defmt.ferrous-systems.com/) and the logs will show a message when the
button is pressed and another from the init function again, showing that the device has reset and is booting again.

The example has been tested on a [ST Nucleo-F401RE](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) development
board but should work on any STM32F4xx family microcontroller as long as a button (pulled high by default) is connected
on PC13 (or the code is adapted to an alternative port).

## Prerequisites
1. [Install Rust](https://www.rust-lang.org/tools/install)
1. Optional: ensure that the rust toolchain is up-to-date: `rustup update`
1. Install [`probe-run`](https://crates.io/crates/probe-run): `cargo install probe-run`
1. Install [`flip-link`](https://crates.io/crates/flip-link): `cargo install flip-link`
   * Note: `flip-link` is not strictly necessary for this example (it doesn't need
     stack protection), however it can be considered best practices to include it.
1. Install the cross-compile target: `rustup target add thumbv7em-none-eabihf`
1. Optional: install the LLVM tools: `rustup component add llvm-tools-preview`
1. Install the STLink drivers

## Build & Download to Board
1. Connect the board via USB
1. Optional: change your targeted platform in `Cargo.toml` and `.cargo/config` (it defaults to STM32F401RE)
1. Run `cargo run`
1. Enjoy your running program :)
