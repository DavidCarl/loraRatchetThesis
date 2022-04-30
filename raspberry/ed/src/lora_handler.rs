use sx127x_lora::LoRa;
use std::{thread, time};

use rppal::gpio::OutputPin;
use rppal::hal::Delay;
use rppal::spi::Spi;

use crate::filehandling::Config;


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
            thread::sleep(time::Duration::from_millis(config.rx1_delay));
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