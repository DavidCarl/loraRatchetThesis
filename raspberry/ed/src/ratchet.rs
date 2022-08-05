use std::{thread, time};

use rand_core::OsRng;

use doubleratchet::ed::EDRatchet;

use sx127x_lora::LoRa;
use sx127x_lora::RadioMode;

use rppal::gpio::OutputPin;
use rppal::spi::Spi;

use crate::{
    filehandling::{Config},
    lora_handler::{lora_send, recieve_window},
    edhoc::{RatchetKeys}
};

pub fn run(
    lora: &mut LoRa<Spi, OutputPin, OutputPin>,
    ratchetkeys: RatchetKeys,
    dhr_const: u16,
    config: Config,
) {

    let ed_ratchet = EDRatchet::new(
        ratchetkeys.ed_rk.try_into().unwrap(),
        ratchetkeys.ed_rck.try_into().unwrap(),
        ratchetkeys.ed_sck.try_into().unwrap(),
        ratchetkeys.devaddr.clone().try_into().unwrap(),
        OsRng,
    );

    thread::sleep(time::Duration::from_millis(5000));

    loraratchet_main_loop(lora, ed_ratchet, dhr_const, 1, config, ratchetkeys.devaddr);
}

fn loraratchet_main_loop(
    lora: &mut LoRa<Spi, OutputPin, OutputPin>,
    mut ed_ratchet: EDRatchet<OsRng>,
    dhr_const: u16,
    n: i32,
    config: Config,
    _devaddr: Vec<u8>,
) {
    loop {
        println!("{:?}", n);
        let random_message: [u8; 8] = rand::random();
        let uplink = ed_ratchet.ratchet_encrypt_payload(&random_message);
        lora_send(lora, uplink);

        let incoming = recieve_window(lora, config);
        if !incoming.is_empty() {
            match ed_ratchet.receive(incoming.to_vec()) {
                Ok(x) => match x {
                    Some(y) => {
                        println!("receiving message from server {:?}", y)
                    }
                    None => println!("Ratchet step performed"),
                },
                Err(x) => {
                    println!("{:?}", x)
                }
            };
        }
        if ed_ratchet.fcnt_up >= dhr_const {
            let dhr_req = ed_ratchet.initiate_ratch(); 
            lora_send(lora, dhr_req);
                    let incoming = recieve_window(lora, config);
                    if !incoming.is_empty() {
                        match ed_ratchet.receive(incoming.to_vec()) {
                            Ok(x) => match x {
                                Some(y) => {
                                    println!("receiving message from server {:?}", y)
                                }
                                None => println!("test"),
                            },
                            Err(x) => {
                                println!("{:?}", x)
                            }
                        };
                    
                }
        }
        let _ = lora.set_mode(RadioMode::Sleep);
        thread::sleep(time::Duration::from_millis(10000));
    }
}