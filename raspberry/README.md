# Lora Ratchet on Raspberry Pi

## Prerequisites

* ARM Rust toolchain
* Raspberry Pi (We used 3B+)
* SX1276 modules

For the ARM rust toolchain, we used [cross](https://github.com/cross-rs/cross). This requires Docker, but comes in a single package.

## What is in this repo

* [Client](https://github.com/DavidCarl/rasp_lora_ratchet/tree/main/client)
* [Server](https://github.com/DavidCarl/rasp_lora_ratchet/tree/main/server)


## Run

### Wiring

#! TODO <Insert wiring diagram here>

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