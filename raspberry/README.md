# Lora Ratchet on Raspberry Pi

## What is in this repo

This directory holds the source code for the implementations of LoRaRatchet build for raspberry pis. There is also a bash script for easy compile and upload of the binaries to the raspberry pis.

* [as:](https://github.com/DavidCarl/loraRatchetThesis/tree/main/raspberry/as) This is the LoRaRatchet server
* [ed:](https://github.com/DavidCarl/loraRatchetThesis/tree/main/raspberry/ed) This is the LoRaRatchet client

## Prerequisites

* ARM Rust toolchain
* Raspberry Pi (We used 3B+)
* SX1276 modules

For the ARM rust toolchain, we used [cross](https://github.com/cross-rs/cross). This requires Docker, but comes in a single package.

## Run

### Wiring Diagram

![Wiring Diagram](https://github.com/DavidCarl/loraRatchetThesis/blob/main/raspberry/resources/wiring_diagram.png?raw=true)

Wiring in text:
Raspberry Pi <-> SX1276

| Pin | name       |      | Name  |
|-----|------------|------|-------|
| 06  | GPIO22     | <->  | GND   |
| 15  | 3.3        | <->  | RST   |
| 17  | SPI_MOSI   | <->  | VDD   |
| 19  | SPI_MISO   | <->  | MOSI  |
| 21  | SPI_CLK    | <->  | MISO  |
| 23  | SPI_CLK    | <->  | SCK   |
| 24  | SPI_CE0_N  | <->  | NSS   |

## Build

### Automated

By using the included script there are a few prerequisites. You will need the tool called `sshpass`. This is avaible in different package managers, eg. arch https://archlinux.org/packages/community/x86_64/sshpass/

Now you would need to change the following things in the script. server and client `password`, `username` and `ip`. Theses changes should make it so when you run the script, it automaticly compiles and SCP the files to the client and server raspberry pi.

### Manually

Go into eiher the client or server, whatever you want to compile first. 

Change your toolchain to either use a ARM toolchain, or use cross. Here we are using cross for simplicity sake.

`cross build --target arm-unknown-linux-gnueabihf`

This should create a binary in the 

`target/arm-unknown-linux-gnueabihf/debug/rasp_lora_<client or server>`

now transfer this to your raspberry pi.

### Config files

We made config files for the code, all the config files can be found in the respective directories, client & server.

The config files required for the ed and as is located in the directories, and are called `keys.json` for the as and `config.json`, `keys.json` for the ed. All the config files need to be alongside the binary on the raspberry pi for them to load.