use rand_core::{OsRng,RngCore};

use rppal::gpio::OutputPin;
use rppal::spi::Spi;

use ::edhoc::edhoc::{
    api::{Msg1Receiver, Msg3Receiver},
    error::{OwnError, OwnOrPeerError},
    PartyR,
};

use doubleratchet::r#as::ASRatchet;

use x25519_dalek_ng::{PublicKey, StaticSecret};

use sx127x_lora::LoRa;

use std::collections::HashMap;

use crate::{
    filehandler::{load_static_keys, StaticKeys},
    lora_handler::{lora_send},
    phypayload_handler::{prepare_message, unwrap_message},
};

const DEVEUI: [u8; 8] = [1, 1, 2, 3, 4, 5, 6, 7];
const APPEUI: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

pub struct TypeZero {
    pub msg3_receivers: HashMap<[u8; 4], PartyR<Msg3Receiver>>,
    pub lora: LoRa<Spi, OutputPin, OutputPin>,
    pub fcntdownmap: HashMap<[u8; 4], u16>
}

/// Handle the zeroth [[0]] message in the EDHOC handshake, initiate the handshake from a AS point of view with a new ED. This function handle all the calls to the different libraries. 
/// It will also transmit the response in this handshake, which is the oneth [[1]] message.
///     
/// # Arguments
///
/// * `buffer` - The incomming message
/// * `msg3_recievers` - A hashmap where the reciever object needs to be stored based on a devaddr
/// * `lora` - A sx127x object, used for broadcasting a response
/// * `as_static_material` - Our static key material. Used to generate our staticSecret
pub fn handle_m_type_zero(
    buffer: Vec<u8>,
    mut msg3_receivers: HashMap<[u8; 4], PartyR<Msg3Receiver>>,
    mut lora: LoRa<Spi, OutputPin, OutputPin>,
    as_static_material: [u8; 32],
    mut fcntdownmap: HashMap<[u8; 4], u16>
) -> TypeZero {
    let phypayload = unwrap_message(buffer, true);//unpack_edhoc_first_message(buffer);

    let msg = phypayload.msg;

    let as_static_priv = StaticSecret::from(as_static_material);
    let as_static_pub = PublicKey::from(&as_static_priv);

    let as_kid = [0xA3].to_vec();

    let mut as_ephemeral_keying = [0u8;32];
    OsRng.fill_bytes(&mut as_ephemeral_keying);
    

    let msg1_receiver = PartyR::new(as_ephemeral_keying, as_static_priv, as_static_pub, as_kid);
    let res = handle_first_gen_second_message(msg.to_vec(), msg1_receiver);
    match res {
        Ok(rtn) => {
            msg3_receivers.insert(rtn.devaddr, rtn.msg3_receiver);
            lora_send(&mut lora,rtn.msg);
            fcntdownmap.insert(rtn.devaddr, 1);
        }
        Err(error) => match error {
            OwnOrPeerError::OwnError(x) => {
                lora_send(&mut lora, x);
            }
            OwnOrPeerError::PeerError(x) => {
                println!("Error in m_type_zero {:?}", x)
            }
        },
    }
    return TypeZero {
        msg3_receivers,
        lora,
        fcntdownmap
    }
}

pub struct TypeTwo {
    pub msg3_receivers: HashMap<[u8; 4], PartyR<Msg3Receiver>>,
    pub connections: HashMap<[u8; 4], ASRatchet<OsRng>>,
    pub lora: LoRa<Spi, OutputPin, OutputPin>,
    pub fcntdownmap: HashMap<[u8; 4], u16>
}

/// handle the second [[2]] message in the EDHOC handshake, and transmit the third [[3]] message in the sequence.
///     
/// # Arguments
///
/// * `buffer` - The incomming message
/// * `msg3_receivers` - A hashmap where the reciever object needs to be stored based on a devaddr
/// * `connections` - A hashmap where the ratchet object needs to be stored based on a devaddr
/// * `lora` - A sx127x object, used for broadcasting a response
pub fn handle_m_type_two(
    buffer: Vec<u8>,
    mut msg3_receivers: HashMap<[u8; 4], PartyR<Msg3Receiver>>,
    mut connections: HashMap<[u8; 4], ASRatchet<OsRng>>,
    mut lora: LoRa<Spi, OutputPin, OutputPin>,
    mut fcntdownmap: HashMap<[u8; 4], u16>
) -> TypeTwo {
    let phypayload = unwrap_message(buffer, false);//unpack_edhoc_message(buffer);
    let msg = phypayload.msg;
    let devaddr = phypayload.devaddr;
    let msg3rec = msg3_receivers.remove(&devaddr).unwrap();

    let payload = handle_third_gen_fourth_message(msg.to_vec(), msg3rec);
    
    let fcntdown_op = fcntdownmap.remove(&devaddr);
    let fcntdown = match fcntdown_op {
        Some(number) => number,
        None => 0,
    };
    
    match payload {
        Ok(msg4) => {
            let (msg, fcntdown) = prepare_message(msg4.msg4_bytes, 3, Some(devaddr), fcntdown);
            lora_send(&mut lora, msg);

            let as_ratchet = ASRatchet::new(
                msg4.as_master.try_into().unwrap(),
                msg4.as_rck.try_into().unwrap(),
                msg4.as_sck.try_into().unwrap(),
                devaddr,
                OsRng,
            );
            connections.insert(devaddr, as_ratchet);
            fcntdownmap.insert(devaddr, fcntdown);
        }
        Err(error) => match error {
            OwnOrPeerError::OwnError(x) => {
                lora_send(&mut lora, x);
            }
            OwnOrPeerError::PeerError(x) => {
                println!("Error in m_type_two {:?}", x)
            }
        },
    }


    TypeTwo {
        msg3_receivers,
        connections,
        lora,
        fcntdownmap
    }
}


struct Msg2 {
    msg: Vec<u8>,
    msg3_receiver: PartyR<Msg3Receiver>,
    devaddr: [u8; 4],
}

/// This function handles the EDHOC logic behind the first [[0]] message. It generates the second message, and the object we need to verify the third [[3]] message later, so we can make
/// sure it comes from the right ED. We also generate a devaddr we use for identifying the devices.
///     
/// # Arguments
///
/// * `msg` - the message which needs to be handled.
/// * `msg1_receiver` - Verifier object, so we can start the whole EDHOC verification.
fn handle_first_gen_second_message(
    msg: Vec<u8>,
    msg1_receiver: PartyR<Msg1Receiver>,
) -> Result<Msg2, OwnOrPeerError> {
    let (msg2_sender, deveui, appeui) = match msg1_receiver.handle_message_1_ead(msg) {
        Err(OwnError(b)) => {
            return Err(OwnOrPeerError::OwnError(b));
        }
        Ok(val) => val,
    };

    if appeui.unwrap() != APPEUI {
        return Err(OwnOrPeerError::PeerError("Wrong APPEUI".to_string()));
    }
    println!("{:?} {:?}", deveui, DEVEUI);
    if deveui != DEVEUI {
        return Err(OwnOrPeerError::PeerError("Wrong DEVEUI".to_string()));
    }

    let (msg2_bytes, msg3_receiver) =  msg2_sender.generate_message_2(APPEUI.to_vec(), None)?;
    println!("{:?}", msg2_bytes);
    let devaddr: [u8; 4] = rand::random();
    
    let (msg, _fcntdown) = prepare_message(msg2_bytes, 1, Some(devaddr), 0);


    Ok(Msg2 {
        msg,
        msg3_receiver,
        devaddr,
    })
}

struct Msg4 {
    msg4_bytes: Vec<u8>,
    as_sck: Vec<u8>,
    as_rck: Vec<u8>,
    as_master: Vec<u8>,
}

/// This function handles the EDHOC logic behind the third [[3]] message. It extracts a KID value, which makes us able to load pre-known keys from a file. 
/// We then use these informations to get the keys we need to start our LoRaRatchet protocol and send the fourth [[4]] message.
///     
/// # Arguments
///
/// * `msg` - the message which needs to be handled.
/// * `msg3_receiver` - Verifier object, so we can continue the whole EDHOC verification.
fn handle_third_gen_fourth_message(
    msg: Vec<u8>,
    msg3_receiver: PartyR<Msg3Receiver>,
) -> Result<Msg4, OwnOrPeerError> {
    let (msg3verifier, ed_kid) = msg3_receiver.unpack_message_3_return_kid(msg)?;

    let enc_keys: StaticKeys = load_static_keys("./keys.json".to_string());
    let mut opt_ed_static_pub: Option<PublicKey> = None;

    // looking through stored ED keys, and tried to find a match
    for each in enc_keys.ed_keys {
        if each.kid.to_vec() == ed_kid {
            opt_ed_static_pub = Some(PublicKey::from(each.ed_static_material));
            println!("{:?}", opt_ed_static_pub)
        }
    }
    match opt_ed_static_pub {
        Some(ed_static_pub) => {
            let (msg4_sender, as_sck, as_rck, as_master) = msg3verifier.verify_message_3(ed_static_pub.as_bytes())?;
            let msg4_bytes =  msg4_sender.generate_message_4(None)?;

            return Ok(Msg4 {
                msg4_bytes,
                as_sck,
                as_rck,
                as_master,
            })
            
        }
        None => panic!("Missing kid value"),
    }
}
