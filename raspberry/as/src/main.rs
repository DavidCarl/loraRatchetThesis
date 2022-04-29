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
mod generics;

const LORA_CS_PIN: u8 = 8;
const LORA_RESET_PIN: u8 = 22;
const FREQUENCY: i64 = 915;

fn main() {
    let lora = setup_sx127x(125000, 7);
    main_loop(lora);
}

/// This function creates a sx127x object, which enables us to send and recieve messages by
/// using the sx1276 lora module.
///
/// # Arguments
///
/// * `bandwith` - Sets the signal bandwith of the module. supported values are `800` Hz, `10400` Hz, `15600` Hz, `20800` Hz, `31250` Hz, `41700` Hz, `62500` Hz, `125000` Hz and `250000` Hz
/// * `spreadfactor` - Sets the spreading factor of the radio. Supported values are between 6 and 12. If a spreading factor of 6 is set, implicit header mode must be used to transmit and receive packets.
fn setup_sx127x(bandwidth: i64, spreadfactor: u8) -> LoRa<Spi, OutputPin, OutputPin> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0).unwrap();

    let gpio = Gpio::new().unwrap();

    let cs = gpio.get(LORA_CS_PIN).unwrap().into_output();
    let reset = gpio.get(LORA_RESET_PIN).unwrap().into_output();

    let mut lora = sx127x_lora::LoRa::new(spi, cs, reset, FREQUENCY, &mut Delay).unwrap();
    let _ = lora.set_signal_bandwidth(bandwidth);
    let _ = lora.set_spreading_factor(spreadfactor);
    lora
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
        let poll = lora.poll_irq(None, &mut Delay); //30 Second timeout
        match poll {
            Ok(size) => {
                println!("Recieved packet with size: {:?}", size);
                let buffer = lora.read_packet().unwrap(); // Received buffer. NOTE: 255 bytes are always returned
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
                let (msg_buffer, len) = generics::get_message_length(newout);
                let transmit = lora.transmit_payload_busy(msg_buffer, len);
                match transmit {
                    Ok(packet_size) => {
                        println!("Sent packet with size: {:?}", packet_size)
                    }
                    Err(_) => println!("Error"),
                }
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