extern crate alloc;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use std::convert::TryInto;
use std::{thread, time::*};

use twoRatchet::AS::ASRatchet;
use twoRatchet::ED::EDRatchet;
//use embedded_hal::digital::v1::OutputPin;
const DHR_CONST: u16 = 64;

mod edhoc;
mod esp32;

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    /*
    Parti I generate message 1
    */

    let (ed_sck, ed_rck, rk_ed, as_sck, as_rck, as_rk, devaddr) = edhoc::join();

    let hrng = esp32::HRNG;

    let hrng1 = esp32::HRNG;

    let mut ed_ratchet = EDRatchet::new(
        rk_ed.try_into().unwrap(),
        ed_rck.try_into().unwrap(),
        ed_sck.try_into().unwrap(),
        devaddr.clone(),
        hrng,
    );

    let mut as_ratchet = ASRatchet::new(
        as_rk.try_into().unwrap(),
        as_rck.try_into().unwrap(),
        as_sck.try_into().unwrap(),
        devaddr.clone(),
        hrng1,
    );

    loop {
        thread::sleep(Duration::from_millis(1000));
        let payload = ed_ratchet.ratchet_encrypt_payload(&[2; 3]);

        match as_ratchet.receive(payload) {
            Ok((x, _b)) => println!("AS recevied message {:?}", x),
            Err(_s) => println!("an eror occurred"),
        };
        if DHR_CONST <= ed_ratchet.fcnt_up {
            let dhr_req = ed_ratchet.initiate_ratch();
            let dh_ack = match as_ratchet.receive(dhr_req) {
                Ok((x, _b)) => x,
                Err(_s) => continue,
            };
            let _ = ed_ratchet.receive(dh_ack);
        }
    }
}
