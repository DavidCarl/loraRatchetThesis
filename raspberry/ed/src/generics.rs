use sx127x_lora::LoRa;
use std::{thread, time};

use rppal::gpio::OutputPin;
use rppal::hal::Delay;
use rppal::spi::Spi;

use crate::filehandling::Config;


pub fn get_message_lenght(message: Vec<u8>) -> ([u8; 255], usize) {
    let mut buffer = [0; 255];
    for (i, byte) in message.iter().enumerate() {
        buffer[i] = *byte;
    }
    (buffer, message.len())
}

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