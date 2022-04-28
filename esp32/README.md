# ESP32

## Prerequisites

* ESP32 Rust toolchain
* Raspberry Pi (We used 3B+)
* SX1276 modules

For the ESP32 rust toolchain, we used a toolchain made by esp-rs, and it can be found [here](https://github.com/esp-rs/rust-build).

## Running

### Compile

You need to set some envoirment variables, for the ESP32 to connect to wifi. 

`RUST_ESP32_STD_DEMO_WIFI_SSID=<ssid>`
`RUST_ESP32_STD_DEMO_WIFI_PASS=<password>`

When this is in place, you have the ability to build for the ESP32, do this by entering the directory and run `cargo build`.

This will compile a binary in the `target/debug` directory, this file should now be flashed unto the ESP32.

### Flash to ESP32

For flashing the binary unto the ESP32, we used a tool called `espflash`, this can be installed by running `cargo install espflash`, and you should be able to call it with `cargo espflash`. A example of how to flash one of the projects would be like this `cargo espflash /dev/ttyUSB0 target/debug/[binaryhere]`

Now you should be able to power up the ESP32 and you should have the application running.

**NOTE** Some ESP32 requires you to ground pin [insert pin here]