use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
extern crate alloc;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::{ sync::Arc, thread};

use core::convert::TryInto;

use embedded_svc::httpd::*;

use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sysloop::*;


use doubleratchet::ed::EDRatchet;
use wifihandler::wifi;

use crate::hrng::HRNG;
mod edhoc;
mod hrng;
mod wifihandler;
const DHR_CONST: u16 = 256;


fn main() -> Result<()> {
    // initialize wifi stack
    esp_idf_sys::link_patches();
    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    #[allow(clippy::redundant_clone)]
    #[allow(unused_mut)]
    let mut wifi = wifi(
        netif_stack,
        sys_loop_stack,
        default_nvs,
    )?;

    // open tcp stream

    match TcpStream::connect("192.168.1.227:8888") {
        Ok(mut stream) => handle_connection(&mut stream),
        Err(e) => {
            panic!("Could not connect to server {}", e);
        }
    }?;

    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), Error> {

    // perform join procedure
    let (ed_sck, ed_rck, ed_rk, devaddr) = match edhoc::join_procedure(stream) {
        Some(join_output) => join_output,
        None => return Ok(()),
    };
    // initialize ratchet
    let mut ratchet = EDRatchet::new(
        ed_rk.try_into().unwrap(),
        ed_rck.try_into().unwrap(),
        ed_sck.try_into().unwrap(),
        devaddr.try_into().unwrap(),
        HRNG,
    );

    // running continous communications, with a 1 second thread sleep
    // For every iteration, a uplink message is sent, and the
    stream
        .set_read_timeout(Some(Duration::from_millis(5000)))?;
    loop {
        thread::sleep(Duration::from_millis(1000));
        let uplink = ratchet.ratchet_encrypt_payload(b"uplink");
        stream.write_all(&uplink)?;
        stream.flush()?;

        if ratchet.fcnt_up >= DHR_CONST {
            let dhr_req = ratchet.initiate_ratch();
            stream.write_all(&dhr_req)?;
            stream.flush()?;
            let mut buf = [0; 64];
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes) => bytes,
                _ => continue,
            };
            let dhr_ack = &buf[0..bytes_read];
            match ratchet.receive(dhr_ack.to_vec()) {
                Ok(x) => match x {
                    Some(x) => println!("receiving message from server {:?}", x),
                    None => continue,
                },
                Err(s) => {
                    println!("error during receive {}", s);
                    continue;
                }
            };
        } else {
            // if we do not want to send a DHReq, then we'll just listen for a message
            let mut buf = [0; 64];
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes) => bytes,
                _ => continue,
            };
            let downlink = &buf[0..bytes_read]; // if this is not the dhrack, it will still be decrypted and handled
            match ratchet.receive(downlink.to_vec()) {
                Ok(x) => match x {
                    Some(x) => println!("receiving message from server {:?}", x),
                    None => continue,
                },
                Err(s) => {
                    println!("error during receive {}", s);
                    continue;
                }
            };
        }
    }
}

