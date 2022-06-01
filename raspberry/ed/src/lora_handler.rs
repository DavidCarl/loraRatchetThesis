use sx127x_lora::{LoRa, RadioMode};
use std::{thread, time};
use rppal::gpio::{Gpio, OutputPin};

use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

extern crate linux_embedded_hal as hal;

use crate::filehandling::Config;
const LORA_CS_PIN: u8 = 8;
const LORA_RESET_PIN: u8 = 22;
const FREQUENCY: i64 = 868;

// EUI64 mac addresser - for appeui og deveui

/// This function creates a sx127x object, which enables us to send and recieve messages by
/// using the sx1276 lora module.
///
/// # Arguments
///
/// * `bandwith` - Sets the signal bandwith of the module. supported values are `800` Hz, `10400` Hz, `15600` Hz, `20800` Hz, `31250` Hz, `41700` Hz, `62500` Hz, `125000` Hz and `250000` Hz
/// * `spreadfactor` - Sets the spreading factor of the radio. Supported values are between 6 and 12. If a spreading factor of 6 is set, implicit header mode must be used to transmit and receive packets.
pub fn setup_sx127x(bandwidth: i64, spreadfactor: u8) -> LoRa<Spi, OutputPin, OutputPin> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0).unwrap();

    let gpio = Gpio::new().unwrap();

    let cs = gpio.get(LORA_CS_PIN).unwrap().into_output();
    let reset = gpio.get(LORA_RESET_PIN).unwrap().into_output();

    let mut lora = sx127x_lora::LoRa::new(spi, cs, reset, FREQUENCY, &mut Delay).unwrap();
    let _ = lora.set_tx_power(17,1);
    let _ = lora.set_signal_bandwidth(bandwidth);
    let _ = lora.set_spreading_factor(spreadfactor);
    lora
}

fn get_message_length(message: Vec<u8>) -> ([u8; 255], usize) {
    let mut buffer = [0; 255];
    for (i, byte) in message.iter().enumerate() {
        buffer[i] = *byte;
    }
    (buffer, message.len())
}
/// Opens two receive windows
/// 
///     
/// # Arguments
///
/// * `lora` - mutably borrowed lora object
/// * `config` - Object containing configuration options, such as receive window delay time
pub fn recieve_window(lora: &mut LoRa<Spi, OutputPin, OutputPin>, config: Config) -> Vec<u8> {
    //Result<ReceiveWindow, Box<dyn stdError>> {
    thread::sleep(time::Duration::from_millis(config.rx1_delay));
    let poll = lora.poll_irq(Some(config.rx1_duration), &mut Delay);
    match poll {
        Ok(size) => {
            let buffer = lora.read_packet().unwrap();
            println!("Recieved packet with size: {:?}", size);
            buffer
        }
        Err(_) => {
            println!("Putting in sleep mode");
            let _ = lora.set_mode(RadioMode::Sleep);
            thread::sleep(time::Duration::from_millis(config.rx2_delay));
            println!("Waking up from sleep mode");
            let poll = lora.poll_irq(Some(config.rx1_duration), &mut Delay);
            match poll {
                Ok(size) => {
                    let buffer = lora.read_packet().unwrap();
                    println!("Recieved packet with size: {:?}", size);
                    buffer
                }
                Err(_) => Vec::new(),
            }
        }
    }
}


    /// Wrapper function for sending message over lora object
    ///
    /// # Arguments
    ///
    /// * `lora` - mutable referenced to our lora module
    ///
    /// # Note 
    /// It is important to notice that transmitting may fail, this may require a rerun of the edhoc handshake
    /// if one of those messages fail
pub fn lora_send(lora: &mut LoRa<Spi, OutputPin, OutputPin>, message : Vec<u8>)  {

    let (msg_buffer, len) = get_message_length(message);
    let transmit = lora.transmit_payload_busy(msg_buffer, len);
    match transmit {
        Ok(packet_size) => {
            println!("Sent packet with size: {:?}", packet_size)
        }
        Err(_) => println!("Transmission Error"),
    }

}