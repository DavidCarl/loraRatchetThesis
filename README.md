# LoraRatchet Thesis implementations

## What is in this repo

This repo contains applications (server & client) for implemented LoraRatchet on either Raspberry pis or ESP32.

* [esp32](https://github.com/DavidCarl/loraRatchetThesis/tree/main/esp32)
* [raspberry pi](https://github.com/DavidCarl/loraRatchetThesis/tree/main/raspberry)

## Run

If you want to run the thesis projects on either a Raspberry pi og ESP32, there are applications and code examples for those. Please go into their respective directory named such.

## Modified libraries

We modified several libraries to get this working. This is both 

---

oscore: [original](https://github.com/martindisch/oscore) - [modified](https://github.com/DavidCarl/oscore)

Here we had to update the library to a newer version. 

---

sx127x_lora: [original](https://crates.io/crates/sx127x_lora) - [modified](https://github.com/DavidCarl/sx127x_lora)

Here we modified some ...

---