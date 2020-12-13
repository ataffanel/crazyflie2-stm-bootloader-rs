# Crazyflie 2.x STM32 bootloader in Rust

Experimental re-implementation of the [crazyflie2-stm-bootloader](https://github.com/bitcraze/crazyflie2-stm-bootloader)
written in Rust.

This firmware currently implements just enough commands to be able to flash the Crazyflie when running "make cload" in the crazyflie firmware project.
The goal is to implement a working bootloader replacement, mostly as a way to learn Rust on embedded systems.

## Compiling and running

A debug probe connected to the Crazyflie using the debug adapter board is required to work on the bootloader.
An ST-Link v2 works, any probe supported by probe-run should work fine.

Compiling and running requires rust, probe-run and flip-link.
It is assumed that rust has been installed using [rustup](https://rustup.rs).

On ubuntu/Debian/Raspberry pi OS the required dependencies can be installed with (openocd is only required to install udev rules for common jtag/swd probes):
```
sudo apt install build-essential libusb-1.0-0-dev pkg-config openocd
```

On Mac and Windows no extra dependencies should be needed.

The required rust tool and target can be installed with:
```
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
