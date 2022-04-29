use sx127x_lora::LoRa;
use std::{thread, time};

use rppal::gpio::OutputPin;
use rppal::spi::Spi;


/// Converts a Vector to an array of a fixed size and length of the given Vector, needed to feed vectors to lora driver
///
/// # Arguments
///
/// * `message` - The messages which we need to convert and get length
fn get_message_length(message: Vec<u8>) -> ([u8; 255], usize) {
    let mut buffer = [0; 255];
    for (i, byte) in message.iter().enumerate() {
        buffer[i] = *byte;
    }
    (buffer, message.len())
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