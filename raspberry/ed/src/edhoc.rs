use sx127x_lora::LoRa;

use rppal::gpio::OutputPin;
use rppal::spi::Spi;
use x25519_dalek_ng::{StaticSecret, PublicKey};

use ::edhoc::edhoc::{
    api::{Msg1Sender, Msg2Receiver, Msg4ReceiveVerify},
    error::{OwnError, OwnOrPeerError},
    PartyI,
};

use std::fmt;
use std::{error::Error as stdError, result::Result};


use rand_core::{OsRng, RngCore};

use crate::{
    filehandling::{load_static_keys, StaticKeys, Config},
    lora_handler::{recieve_window,lora_send},
    phypayload_handler::{ prepare_message, unwrap_message}
};

const SUITE_ED: u8 = 0;
const METHOD_TYPE_ED: u8 = 3;

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl stdError for MyError {}

pub struct RatchetKeys {
    pub ed_sck: Vec<u8>,
    pub ed_rck: Vec<u8>,
    pub ed_rk: Vec<u8>,
    pub devaddr: Vec<u8>,
}

/// This functions starts the handshake for the ED. 
/// 
/// # Arguments
/// 
/// * `lora` - This is the active lora module
/// * `enc_keys` - This is our StaticKeys struct, theses keys are used for parts of the handshake.
/// * `deveui` - This devEUI, this is
/// * `appeui` - The appEUI, this should be the same for the ED and AS
/// * `config` - The application configuration
/// 
pub fn handshake(
    lora: &mut LoRa<Spi, OutputPin, OutputPin>,
    enc_keys: StaticKeys,
    deveui: [u8; 8],
    appeui: [u8; 8],
    config: Config,
) -> Result<RatchetKeys, Box<dyn stdError>> {
    let ed_kid = [0xA2].to_vec();
    let ed_static_priv = StaticSecret::from(enc_keys.ed_static_material);
    let ed_static_pub = PublicKey::from(&ed_static_priv);


    let mut ed_ephemeral_keying = [0u8;32];
    OsRng.fill_bytes(&mut ed_ephemeral_keying);

    let msg1_sender = PartyI::new(
        deveui.to_vec(),
        Some(appeui.to_vec()),
        ed_ephemeral_keying,
        ed_static_priv,
        ed_static_pub,
        ed_kid,
    );
    let (payload1, msg2_reciever) = edhoc_first_message(msg1_sender);
    lora_send(lora, payload1);


    let incoming = recieve_window(lora, config);
    match incoming[0] {
        1 => match edhoc_third_message(incoming.to_vec(), msg2_reciever) {
            Ok((msg3, msg4_reciever)) => {

                lora_send(lora, msg3);

                let incoming = recieve_window(lora, config);
                match incoming[0] {
                    3 => {

                        let rtn = handle_message_fourth(incoming, msg4_reciever);
                        match rtn {
                            Ok(values) => {
                                let ratchet_keys = RatchetKeys {
                                    ed_sck: values.ed_sck,
                                    ed_rck: values.ed_rck,
                                    ed_rk: values.ed_rk,
                                    devaddr: values.devaddr,
                                };
                                Ok(ratchet_keys)
                            }
                            Err(OwnOrPeerError::OwnError(x)) => {
                                lora_send(lora, x);
                                Err(Box::new(MyError("Own error in m_type 3".to_string())))
                            }
                            Err(OwnOrPeerError::PeerError(x)) => {
                                println!("Got peer error {:?}", x);
                                Err(Box::new(MyError("Peer error in m_type 3".to_string())))
                            }
                        }
                    }
                    _ => Err(Box::new(MyError(
                        "Wrong order, got some other message than mtype 3".to_string(),
                    ))),
                }
            }
            Err(OwnOrPeerError::PeerError(x)) => Err(Box::new(MyError(x))),
            Err(OwnOrPeerError::OwnError(x)) => {
                lora_send(lora, x);
                return Err(Box::new(MyError("Own error".to_string())));
            },
        },
        _ => Err(Box::new(MyError(
            "Recieved nothing in our allocated time span".to_string(),
        ))),
    }
}

/// This functions generates the very first message for the handhake necessary in the EDHOC handshake, and 
/// generates the necessecary information to verify the next message.
/// 
/// # Arguments
/// 
/// * `msg1_sender` - This is needed to generate the first message correctly
/// 
fn edhoc_first_message(msg1_sender: PartyI<Msg1Sender>) -> (Vec<u8>, PartyI<Msg2Receiver>) {
    let (msg1_bytes, msg2_receiver) =
    msg1_sender.generate_message_1(METHOD_TYPE_ED, SUITE_ED).unwrap();

    let payload1 = prepare_message(msg1_bytes, 0,  None);
    (payload1, msg2_receiver)
}

fn edhoc_third_message(
    msg2: Vec<u8>,
    msg2_receiver: PartyI<Msg2Receiver>,
) -> Result<(Vec<u8>, PartyI<Msg4ReceiveVerify>), OwnOrPeerError> {
    let msg_struc = unwrap_message(msg2);

    println!("{:?}", msg_struc.msg.len());
    println!("{:?}", msg_struc.msg);

    let (as_kid, _ad_r, msg2_verifier) = msg2_receiver.unpack_message_2_return_kid(msg_struc.msg)?;

    let enc_keys: StaticKeys = load_static_keys("./keys.json".to_string());
    let mut opt_as_static_pub: Option<PublicKey> = None;
    for each in enc_keys.as_keys {
        if each.kid.to_vec() == as_kid {
            opt_as_static_pub = Some(PublicKey::from(each.as_static_material));
            println!("{:?}", opt_as_static_pub)
        }
    }

    match opt_as_static_pub {
        Some(as_static_pub) => {
            println!("Hello");
            let msg3_sender =
                match msg2_verifier.verify_message_2(as_static_pub.as_bytes()) {
                    Err(OwnError(b)) => {
                        println!("Hello verify");
                        return Err(OwnOrPeerError::OwnError(b));
                    }
                    Ok(val) => val,
                };

            let (msg4_receiver_verifier, msg3_bytes) = match msg3_sender.generate_message_3(None) {
                Err(OwnError(b)) => {
                    println!("Hello");
                    return Err(OwnOrPeerError::OwnError(b));
                }
                Ok(val) => val,
            };

            let payload3 = prepare_message(msg3_bytes, 2, Some(msg_struc.devaddr));
            Ok((payload3, msg4_receiver_verifier))
        }
        None => Err(OwnOrPeerError::OwnError(b"No key found for identity".to_vec())),
    }
}

struct FourthMessage {
    ed_sck: Vec<u8>,
    ed_rck: Vec<u8>,
    ed_rk: Vec<u8>,
    devaddr: Vec<u8>,
}

fn handle_message_fourth(
    msg: Vec<u8>,
    msg4_receiver_verifier: PartyI<Msg4ReceiveVerify>,
) -> Result<FourthMessage, OwnOrPeerError> {
    let msg_struct = unwrap_message(msg);
    let (ed_sck, ed_rck, ed_rk) = msg4_receiver_verifier.handle_message_4(msg_struct.msg)?;
    Ok(FourthMessage{ed_sck, ed_rck, ed_rk, devaddr: msg_struct.devaddr.to_vec()})

}

