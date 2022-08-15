extern crate alloc;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use x25519_dalek_ng::{PublicKey, StaticSecret};

use crate::hrng::HRNG;


use ::edhoc::edhoc::{
    PartyI, PartyR,
};
use doubleratchet::ed::EDRatchet;
use doubleratchet::r#as::ASRatchet;

use embedded_svc::sys_time::SystemTime;



use esp_idf_svc::systime::EspSystemTime;

mod hrng;



pub const I_EPHEMEREAL_SK : [u8;32] = [0xB3,0x11,0x19,0x98,0xCB,0x3F,0x66,0x86,0x63,0xED,0x42,0x51,
                            0xC7,0x8B,0xE6,0xE9,0x5A,0x4D,0xA1,0x27,0xE4,0xF6,0xFE,0xE2,
                            0x75,0xE8,0x55,0xD8,0xD9,0xDF,0xD8,0xED];

pub const R_EPHEMEREAL_SK : [u8;32] = [0xBD,0x86,0xEA,0xF4,0x06,0x5A,0x83,0x6C,0xD2,0x9D,0x0F,0x06,
                            0x91,0xCA,0x2A,0x8E,0xC1,0x3F,0x51,0xD1,0xC4,0x5E,0x1B,0x43,0x72,
                            0xC0,0xCB,0xE4,0x93,0xCE,0xF6,0xBD];

pub const I_STATIC_SK : [u8;32] = [0xCF,0xC4,0xB6,0xED,0x22,0xE7,0x00,0xA3,0x0D,0x5C,0x5B,
                            0xCD,0x61,0xF1,0xF0,0x20,0x49,0xDE,0x23,0x54,0x62,0x33,
                            0x48,0x93,0xD6,0xFF,0x9F,0x0C,0xFE,0xA3,0xFE,0x04];
pub const I_STATIC_PK : [u8;32] = [0x4A,0x49,0xD8,0x8C,0xD5,0xD8,0x41,0xFA,0xB7,0xEF,0x98,
                            0x3E,0x91,0x1D,0x25,0x78,0x86,0x1F,0x95,0x88,0x4F,0x9F,0x5D,
                            0xC4,0x2A,0x2E,0xED,0x33,0xDE,0x79,0xED,0x77];
pub const R_STATIC_SK : [u8;32] = [0x52,0x8B,0x49,0xC6,0x70,0xF8,0xFC,0x16,0xA2,0xAD,0x95,
                                    0xC1,0x88,0x5B,0x2E,0x24,0xFB,0x15,0x76,0x22,0x72,0x79,
                                    0x2A,0xA1,0xCF,0x05,0x1D,0xF5,0xD9,0x3D,0x36,0x94];

pub const R_STATIC_PK : [u8;32]= [0xE6,0x6F,0x35,0x59,0x90,0x22,0x3C,0x3F,0x6C,0xAF,0xF8,
                        0x62,0xE4,0x07,0xED,0xD1,0x17,0x4D,0x07,0x01,0xA0,0x9E,
                        0xCD,0x6A,0x15,0xCE,0xE2,0xC6,0xCE,0x21,0xAA,0x50];


pub const MSG1 : [u8; 56]= [3, 0, 88, 32, 58, 169, 235, 50, 1, 179, 54, 123, 140, 139, 227, 141, 145, 229, 122, 43, 67, 62, 103, 136, 140, 134, 210, 172, 0, 106, 82, 8, 66, 237, 80, 55, 72, 1, 1, 2, 3, 2, 4, 5, 7, 74, 1, 72, 0, 1, 2, 3, 4, 5, 6, 7];
pub const MSG2 : [u8; 54] =  [88, 43, 37, 84, 145, 176, 90, 57, 137, 255, 45, 63, 254, 166, 32, 152, 170, 181, 124, 22, 15, 41, 78, 217, 72, 1, 139, 65, 144, 247, 209, 97, 130, 78, 128, 201, 78, 209, 162, 152, 175, 167, 147, 24, 130, 72, 0, 1, 2, 3, 4, 5, 6, 7];
pub const MSG3 :[u8; 20]= [83, 137, 199, 176, 205, 118, 70, 96, 152, 174, 94, 43, 21, 128, 212, 95, 156, 183, 206, 147];
pub const MSG4 : [u8;9]= [72, 24, 231, 31, 142, 53, 181, 161, 223];
                        
pub const DEVEUI : [u8;8] = [0x1,1,2,3,2,4,5,7];
pub const APPEUI : [u8;8] = [0,1,2,3,4,5,6,7];
pub const KID_I : [u8;1] = [5];
pub const KID_R : [u8;1] = [0x10];
pub const DEVADDR : [u8;4] = [1, 2, 3, 2];


pub const  SK : [u8;32] = [
    16, 8, 7, 78, 159, 104, 210, 58, 89, 216, 177, 79, 10, 252, 39, 141, 8, 160, 148, 36, 29,
    68, 31, 49, 89, 67, 233, 53, 16, 210, 28, 207,
];
pub const DOWNLINK : [u8;32] = [
    0, 171, 247, 26, 19, 92, 119, 193, 156, 216, 49, 89, 90, 174, 165, 23, 124, 247, 30, 79,
    73, 164, 55, 63, 178, 39, 228, 26, 180, 224, 173, 104,
];
pub const UPLINK : [u8;32] = [
    218, 132, 151, 66, 151, 72, 196, 104, 152, 13, 117, 94, 224, 7, 231, 216, 62, 155, 135, 52,
    59, 100, 217, 236, 115, 100, 161, 95, 8, 146, 123, 146,
];


fn main() {


    let ed_static_priv = StaticSecret::from(I_EPHEMEREAL_SK);
    let ed_static_pub = PublicKey::from(&ed_static_priv);

    let mut buf = [0; 32];

    buf.copy_from_slice(&I_STATIC_SK);
    let i_static_sk = StaticSecret::from(buf);
    let pub_st_i = PublicKey::from(&i_static_sk);    

    let mut buf = [0; 32];

    buf.copy_from_slice(&R_STATIC_SK);
    let r_static_sk = StaticSecret::from(buf);
    let pub_st_r = PublicKey::from(&r_static_sk); 



    let msg1sender =  bench("party_i_build", || PartyI::new(                
        DEVEUI.to_vec(),
        Some(APPEUI.to_vec()),
        I_EPHEMEREAL_SK,
        StaticSecret::from(I_STATIC_SK),
        pub_st_i,
        KID_I.to_vec()));

    let (msg1_bytes, msg2_receiver) = bench("msg1_generate",||msg1sender.generate_message_1(3,0)).unwrap();

    let msg1_receiver =  bench("party_r_build", || PartyR::new(
        R_EPHEMEREAL_SK,
        StaticSecret::from(R_STATIC_SK),
        pub_st_r,
        KID_R.to_vec()));


    let (msg2_sender,deveui,appeui)  = bench("msg1_handle", || msg1_receiver.handle_message_1_ead(msg1_bytes)).unwrap();


    let (msg2_bytes,msg3_receiver) =  bench("msg2_generate",||msg2_sender.generate_message_2(appeui.unwrap(),None)).unwrap();


    let  (_r_kid ,_appeui ,msg2_verifier) =  bench("msg2_extract",|| msg2_receiver.unpack_message_2_return_kid(msg2_bytes)).unwrap();

    let msg3_sender = bench("msg2_verify", ||msg2_verifier.verify_message_2(pub_st_r.as_bytes())).unwrap();


    let (msg4_receiver_verifier, msg3_bytes) = bench("msg3_generate",|| msg3_sender.generate_message_3(None)).unwrap();


    let (msg3verifier, _kid) = bench("msg3_extract",||msg3_receiver.unpack_message_3_return_kid(msg3_bytes)).unwrap();

    let (msg4_sender, as_sck, as_rck, as_rk) = bench("msg3_verify",|| msg3verifier.verify_message_3(pub_st_i.as_bytes())).unwrap();

 
    let msg4_bytes = bench("msg4_generate",|| msg4_sender.generate_message_4(None)).unwrap();
    
    
    let (_ed_sck, _ed_rck,_ed_rk) = bench("msg4_handle",|| msg4_receiver_verifier.handle_message_4(msg4_bytes)).unwrap();


    let mut ed_ratchet = bench("ed_build", ||  EDRatchet::new(SK, DOWNLINK, UPLINK, DEVADDR, HRNG));

    let mut as_ratchet = bench("as_build", || ASRatchet::new(SK, UPLINK, DOWNLINK, DEVADDR, HRNG));

    let ed_ciphertext = bench("ed_encrypt",|| ed_ratchet.ratchet_encrypt_payload(b"Message"));

    let as_ciphertext = bench("as_encrypt", ||as_ratchet.ratchet_encrypt_payload(b"Message"));

    let _ed_decrypted = bench("ed_decrypt", || ed_ratchet.receive(as_ciphertext)).unwrap();

    let _ed_decrypted = bench("as_decrypt", ||as_ratchet.receive(ed_ciphertext).unwrap());

    let ed_dhr_req  = bench("ed_initiate_ratch",|| ed_ratchet.initiate_ratch());

    let as_dhr_ack = bench("as_ratchet", || as_ratchet.receive(ed_dhr_req).unwrap().0);

    let _none = bench("ed_finalize_ratch", || ed_ratchet.receive(as_dhr_ack).unwrap());

    let ed_dhr_ack_uplink = ed_ratchet.ratchet_encrypt_payload(b"Message");

    let _none = bench("as_finalize_ratch",||as_ratchet.receive(ed_dhr_ack_uplink).unwrap());

    // resetting Double ratchet context for skipping tests:

    let mut ed_ratchet = EDRatchet::new(SK, DOWNLINK, UPLINK, DEVADDR, HRNG);

    let mut as_ratchet = ASRatchet::new(SK, UPLINK, DOWNLINK, DEVADDR, HRNG);

    let _skip_message = as_ratchet.ratchet_encrypt_payload(b"skipMessage");
    let ed_ciphertext = as_ratchet.ratchet_encrypt_payload(b"Message");

    let _ed_decrypted = bench("ed_decrypt_skip1", || ed_ratchet.receive(ed_ciphertext)).unwrap();

    let _skip_message = ed_ratchet.ratchet_encrypt_payload(b"skipMessage");
    let as_ciphertext = ed_ratchet.ratchet_encrypt_payload(b"Message");

    let _as_decrypted = bench("ed_decrypt_skip1", || as_ratchet.receive(as_ciphertext)).unwrap();

    let mut ed_ratchet = EDRatchet::new(SK,UPLINK,DOWNLINK, DEVADDR, HRNG);
    let mut as_ratchet = ASRatchet::new(SK, DOWNLINK, UPLINK, DEVADDR, HRNG);
    let old_message = as_ratchet.ratchet_encrypt_payload(b"lostMessage");
    let dhr_req = ed_ratchet.initiate_ratch();
    let dhr_ack = as_ratchet.receive(dhr_req).unwrap().0;
    let none = ed_ratchet.receive(dhr_ack).unwrap();
    assert_eq!(none,None);
    let lost_uplink = ed_ratchet.ratchet_encrypt_payload(b"ack uplink");
    let _none = as_ratchet.receive(lost_uplink).unwrap();

    let _decrypted = bench("ed_old_dhrp", || ed_ratchet.receive(old_message).unwrap());
}

 


pub  fn bench<F,T>(title : &str, f: F) -> T
where
    F:  FnOnce() -> T,
{
  let start  = EspSystemTime{}.now();
  let out = f();
  let stop = EspSystemTime{}.now() - start;
  println!("{} took {:?}", title, stop);
  out
}