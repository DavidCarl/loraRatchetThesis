use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
extern crate alloc;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::{env, sync::Arc, thread};

use core::convert::TryInto;

use embedded_svc::httpd::*;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;

use embedded_svc::wifi::*;

use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;

use esp_idf_svc::wifi::*;

use doubleratchet::ed::EDRatchet;

use rand_core::OsRng;

mod edhoc;

const DHR_CONST: u16 = 256;

const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

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
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

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
        OsRng,
    );

    // running continous communications, with a 1 second thread sleep
    // For every iteration, a uplink message is sent, and the
    stream
        .set_read_timeout(Some(Duration::from_millis(5000)))
        .expect("Could not set a read timeout");
    loop {
        thread::sleep(Duration::from_millis(1000));
        let uplink = ratchet.ratchet_encrypt_payload(&[1; 34]);
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

fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    println!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        println!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        println!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    println!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        println!("Wifi connected");

        ping(&ip_settings)?;
    } else {
        println!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

fn ping(ip_settings: &ipv4::ClientSettings) -> Result<()> {
    println!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        println!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
        main();
    }

    println!("Pinging done");

    Ok(())
}
