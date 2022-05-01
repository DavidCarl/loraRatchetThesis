extern crate alloc;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use std::convert::TryInto;
use std::{thread, time::*};
use crate::hrng::HRNG;

use doubleratchet::r#as::ASRatchet;
use doubleratchet::ed::EDRatchet;
const DHR_CONST: u16 = 64;

mod edhoc;
mod hrng;

fn main() {


    /*
    Parti I generate message 1
    */

    let (ed_sck, ed_rck, rk_ed, as_sck, as_rck, as_rk, devaddr) = edhoc::join();



    let mut ed_ratchet = EDRatchet::new(
        rk_ed.try_into().unwrap(),
        ed_rck.try_into().unwrap(),
        ed_sck.try_into().unwrap(),
        devaddr.clone(),
        HRNG,
    );

    let mut as_ratchet = ASRatchet::new(
        as_rk.try_into().unwrap(),
        as_rck.try_into().unwrap(),
        as_sck.try_into().unwrap(),
        devaddr.clone(),
        HRNG,
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
