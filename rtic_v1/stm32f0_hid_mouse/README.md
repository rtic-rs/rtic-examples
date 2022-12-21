# Rust STM32 blackberry trackball firmware

Getting onboard with Rust for embedded devices is like 1,2,3 (more details on the [Rust embedded book](https://rust-embedded.github.io/book/)):

```shell
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ rustup target add thumbv6m-none-eabi
$ cargo install cargo-embed
```

Flashing with a standard STLink v2 is easy with `cargo-embed`:

```shell
$ cargo embed --release
    Finished release [optimized] target(s) in 0.04s
    Flashing /Users/romanvg/dev/cchs/keyboard_trackball/software/stm32f04/target/thumbv6m-none-eabi/release/stm32f042
     Erasing sectors ✔ [00:00:00] [#################################################################]  23.00KB/ 23.00KB @  28.29KB/s (eta 0s )
 Programming pages   ✔ [00:00:02] [#################################################################]  23.00KB/ 23.00KB @   7.24KB/s (eta 0s )
    Finished in 2.942s
```

More context and possible future extensions [at the original repo for this example](https://github.com/brainstorm/bbtrackball-rs).
