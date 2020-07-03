# STM32L0 Monotonic

In this example we show the use of a custom `rtic::Monotonic` implementation
which uses a 16 bit timer of the `STM32L0` MCU.

## Flashing

First, adjust your MCU in the `features` list of the `stm32l0xx-hal` in
`Cargo.toml`:

    stm32l0xx-hal = { version = "...", features = ["rt", "mcu-STM32L071KBTx"] }

Install cargo-flash:

    cargo install cargo-flash

Connect a serial adapter to UART1 (57600 baud). Then, run:

    cargo flash --chip stm32l071kbtx --release

...and observe the logs in the serial console.
