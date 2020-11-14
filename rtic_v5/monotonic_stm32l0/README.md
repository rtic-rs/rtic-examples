# STM32L0 Monotonic

In this example we show the use of a custom `rtic::Monotonic` implementation
which uses a 16 bit timer of the `STM32L0` MCU.

## Flashing and running

Flashing with a standard STLink v2 is easy with `cargo-embed`:

```shell
$ cargo install cargo-embed
$ cargo embed --release
```

Please review the `.embed.toml` file to change your target IC among other options.
