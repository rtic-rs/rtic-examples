# `RTIC examples`

> Here you can find examples on different aspects of the RTIC scheduler.

## Structure

This repo does have example applications based on RTIC framework for popular hardware platforms (for example nRF series and Bluepill).

`rtic_v0.5` dir is for `RTIC` version `v0.5.x`, `rtic_v1` for `v1.0`.
Each folder does have a full project structure with `README.md`, `Cargo.toml` and everything else needed to get the project to compile.

## Requirements

To run these examples, you need to have working environment as described in [Installing the tools](https://rust-embedded.github.io/book/intro/install.html) chapter of **The Embedded Rust Book**.

Short list:

* Rust and cargo
* Toolchain for your microcontroller
* OpenOCD

## Contributing
New examples are always welcome!

When contributing a new example you must make sure to also add it to `.github/dependabot.yml`. To do so, run `update_dependabot_config.sh` which will update the file for you.

## External examples

Some projects maintain RTIC examples in their own repository. Follow these links to find more RTIC examples.

- The [`teensy4-rs` project](https://github.com/mciantyre/teensy4-rs) maintains `RTIC v1.0` examples that run on the Teensy 4.0 and 4.1.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
