# Crazyflie 2.x STM32 bootloader in Rust

Experimental re-implementation of the [crazyflie2-stm-bootloader](https://github.com/bitcraze/crazyflie2-stm-bootloader)
written in Rust.

## Compiling and running

A debug probe connected to the Crazyflie using the debug adapter board is required to work on the bootloader.
An ST-Link v2 works, any probe supported by probe-run should work fine.

Compiling and running requires rust, probe-run and flip-link.
It is assumed that rust has been installed using [rustup](https://rustup.rs).

On ubuntu the required dependencies can be installed with:
```
sudo apt install build-essential libusb-1.0-0-dev pkg-config

cargo install flip-link
cargo install probe-run

rustup target add thumbv7em-none-eabihf
```

To build, flash and run:
```
cargo run --release
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
