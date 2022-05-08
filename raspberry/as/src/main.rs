extern crate linux_embedded_hal as hal;
extern crate sx127x_lora;

use rand_core::OsRng;

use oscore::edhoc::{api::Msg3Receiver, PartyR};

use doubleratchet::r#as::ASRatchet;

use sx127x_lora::LoRa;

use rppal::gpio::{Gpio, OutputPin};
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use std::collections::HashMap;

mod edhoc;
mod filehandler;
mod lora_handler;
mod phypayload_handler;
const LORA_CS_PIN: u8 = 8;
const LORA_RESET_PIN: u8 = 22;
const FREQUENCY: i64 = 915;

fn main() {
    let lora = lora_handler::setup_sx127x(250000, 7);
    main_loop(lora);
}



/// Starting the server application.
/// This function handles all the logic behind listening & recieving messages.
///
/// # Arguments
///
/// * `lora` - Takes a sx127x lora module object
fn main_loop(mut lora: LoRa<Spi, OutputPin, OutputPin>) {
    // load keys
    let enc_keys: filehandler::StaticKeys =
        filehandler::load_static_keys("./keys.json".to_string());

    // Creating two hashmaps, outside the loop to ensure they are no overwritten on each iteration
    // We do this to make the server function more advanced such it can handle multiple clients at a time
    // and access the correct data based on the clients devaddr.
    let mut msg3_receivers: HashMap<[u8; 4], PartyR<Msg3Receiver>> = HashMap::new();
    let mut connections: HashMap<[u8; 4], ASRatchet<OsRng>> = HashMap::new();
    loop {
        let poll = lora.poll_irq(None, &mut Delay);
        match poll {
            Ok(size) => {
                println!("Recieved packet with size: {:?}", size);
                let buffer = lora.read_packet().unwrap();
                match buffer[0] {
                    0 => {
                        println!("Recieved m type 0");
                        let rtn = edhoc::handle_m_type_zero(
                            buffer,
                            msg3_receivers,
                            lora,
                            enc_keys.as_static_material,
                        );
                        msg3_receivers = rtn.msg3_receivers;
                        lora = rtn.lora;
                    }
                    2 => {
                        println!("Recieved m type 2");
                        let rtn = edhoc::handle_m_type_two(
                            buffer,
                            msg3_receivers,
                            connections,
                            lora,
                        );
                        msg3_receivers = rtn.msg3_receivers;
                        connections = rtn.connections;
                        lora = rtn.lora;
                    }
                    5 => {
                        println!("Recieved m type 5");
                        let incoming = &buffer;
                        let rtn = handle_ratchet_message(
                            incoming.to_vec(),
                            lora,
                            connections,
                        );
                        lora = rtn.lora;
                        connections = rtn.connections;
                    }
                    7 => {
                        println!("Recieved m type 7");
                        let incoming = &buffer;
                        let rtn = handle_ratchet_message(
                            incoming.to_vec(),
                            lora,
                            connections,
                        );
                        lora = rtn.lora;
                        connections = rtn.connections;
                    }
                    _ => {
                        println!("Recieved m type _");
                    }
                }
            }
            Err(_) => println!("Timeout"),
        }
    }
}

struct RatchetMessage {
    lora: LoRa<Spi, OutputPin, OutputPin>,
    connections: HashMap<[u8; 4], ASRatchet<OsRng>>,
}

/// This function handles the incomming ratchet messages, this includes decrypting, and checking if
/// we would need to perform a DHR, to update our keys.
///
/// # Arguments
///
/// * `buffer` - The recieved LoRaRatchet message.
/// * `lora` - Takes a sx127x lora module object.
/// * `lora_ratchet` - A hashmap containing all the ASRatchets.
fn handle_ratchet_message(
    buffer: Vec<u8>,
    mut lora: LoRa<Spi, OutputPin, OutputPin>,
    mut connections: HashMap<[u8; 4], ASRatchet<OsRng>>,
) -> RatchetMessage {
    let incoming = &buffer;
    let devaddr: [u8; 4] = buffer[14..18].try_into().unwrap();
    let ratchet = connections.remove(&devaddr);
    match ratchet {
        Some(mut lora_ratchet) => {
            let (newout, sendnew) = match lora_ratchet.receive(incoming.to_vec()) {
                Ok((x, b)) => (x, b),
                Err(x) => {
                    println!("error has happened {:?}", incoming);
                    println!("Error message {:?}", x);
                    connections.insert(devaddr, lora_ratchet);
                    return RatchetMessage {
                        lora,
                        connections,
                    };
                }
            };
            if sendnew {
               lora_handler::lora_send(&mut lora, newout);
            }
            connections.insert(devaddr, lora_ratchet);
        }
        None => println!("No ratchet on this devaddr"),
    }
    RatchetMessage {
        lora,
        connections,
    }
}