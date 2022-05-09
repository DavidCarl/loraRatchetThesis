use ::edhoc::edhoc::{
    error::{OwnError, OwnOrPeerError},
    PartyI, PartyR,
};

use rand_core::RngCore;

use x25519_dalek_ng::{PublicKey, StaticSecret};

use crate::hrng::HRNG;

const ED_STATIC_MATERIAL: [u8; 32] = [
    154, 31, 220, 202, 59, 128, 114, 237, 96, 201, 18, 178, 29, 143, 85, 133, 70, 32, 155, 41, 124,
    111, 51, 127, 254, 98, 103, 99, 0, 38, 102, 4,
];

const AS_STATIC_MATERIAL: [u8; 32] = [
    245, 156, 136, 87, 191, 59, 207, 135, 191, 100, 46, 213, 24, 152, 151, 45, 141, 35, 185, 103,
    168, 73, 74, 231, 37, 220, 227, 42, 68, 62, 196, 109,
];
const DEVEUI: [u8; 8] = [0x1, 1, 2, 3, 2, 4, 5, 7];
const APPEUI: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

const SUITE_I: u8 = 3;
const METHOD_TYPE_I: u8 = 0;

pub fn join() -> (
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    [u8; 4],
) {
    let ed_static_priv = StaticSecret::from(ED_STATIC_MATERIAL);
    let ed_static_pub = PublicKey::from(&ed_static_priv);

    // AS----------------------------------------------------------------
    // "Generate" an ECDH key pair (this is static, but MUST be ephemeral)
    let mut ed_priv = [0; 32];
    HRNG.fill_bytes(&mut ed_priv);

    let i_kid = [0xA2].to_vec();
    let msg1_sender = PartyI::new(
        DEVEUI.to_vec(),
        Some(APPEUI.to_vec()),
        ed_priv,
        ed_static_priv,
        ed_static_pub,
        i_kid,
    );

    let (msg1_bytes, msg2_receiver) = msg1_sender
        .generate_message_1(METHOD_TYPE_I, SUITE_I)
        .unwrap();
    println!("msg1 {}", msg1_bytes.len());

    /*//////////////////////////////
    /// AS initialize and handle message 1
     *////////////////////////////////////////7

    let as_static_priv = StaticSecret::from(AS_STATIC_MATERIAL);
    let as_static_pub = PublicKey::from(&as_static_priv);

    let as_kid = [0xA3].to_vec();

    let as_priv = [0; 32];
    HRNG.fill_bytes(&mut ed_priv);

    let msg1_receiver = PartyR::new(as_priv, as_static_priv, as_static_pub, as_kid);
    let (msg2_sender, _deveui, _appeui) = match msg1_receiver.handle_message_1_ead(msg1_bytes) {
        Err(OwnError(b)) => {
            panic!("{:?}", b)
        }
        Ok(val) => val,
    };

    let (msg2_bytes, msg3_receiver) = match msg2_sender.generate_message_2(APPEUI.to_vec(), None) {
        Err(OwnOrPeerError::PeerError(s)) => {
            panic!("Received error msg: {}", s)
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            panic!("Send these bytes: {}", hexstring(&b))
        }
        Ok(val) => val,
    };

    let devaddr = [2, 56, 45, 12];

    /*///////////////////////////////////////////////////////////////////////////
    /// Initiator receiving and handling message 2, and then generating message 3, and the rck/sck
    ///////////////////////////////////////////////////////////////////// */
    let (_as_kid, _appeui, msg2_verifier) =
        match msg2_receiver.unpack_message_2_return_kid(msg2_bytes) {
            Err(OwnOrPeerError::PeerError(s)) => {
                panic!("Error during  {}", s)
            }
            Err(OwnOrPeerError::OwnError(b)) => {
                panic!("Send these bytes: {}", hexstring(&b))
            }
            Ok(val) => val,
        };

    let msg3_sender = match msg2_verifier.verify_message_2(&as_static_pub.as_bytes().to_vec()) {
        Err(OwnError(b)) => panic!("Send these bytes: {:?}", &b),
        Ok(val) => val,
    };

    let (msg4_receiver_verifier, msg3_bytes) = match msg3_sender.generate_message_3(None) {
        Err(OwnError(b)) => panic!("Send these bytes: {}", hexstring(&b)),
        Ok(val) => val,
    };

    /*///////////////////////////////////////////////////////////////////////////
    /// Responder receiving and handling message 3, and generating message4 and sck rck
    ///////////////////////////////////////////////////////////////////// */

    let (msg3verifier, _ed_kid) = match msg3_receiver.unpack_message_3_return_kid(msg3_bytes) {
        Err(OwnOrPeerError::PeerError(s)) => {
            panic!("received error {} in message 3, shutting down", s);
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            panic!("sending error {:?}, ", b);
        }
        Ok(val) => val,
    };

    let (msg4_sender, as_sck, as_rck, as_rk) =
        match msg3verifier.verify_message_3(&ed_static_pub.as_bytes().to_vec()) {
            Err(OwnOrPeerError::PeerError(s)) => {
                panic!(
                    "received error {} while verifying message 3, shutting down",
                    s
                );
            }
            Err(OwnOrPeerError::OwnError(b)) => {
                panic!("sending error {:?}, ", b);
            }
            Ok(val) => val,
        };

    let msg4_bytes = match msg4_sender.generate_message_4(None) {
        Err(OwnOrPeerError::PeerError(s)) => {
            panic!("Received error msg: {}", s)
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            panic!("Send these bytes: {}", hexstring(&b))
        }
        Ok(val) => val,
    };

    /*///////////////////////////////////////////////////////////////////////////
    /// ED receiving and handling message 4, and generati  sck and rck. Then all is done
    ///////////////////////////////////////////////////////////////////// */

    let (ed_sck, ed_rck, rk_ed) = match msg4_receiver_verifier.handle_message_4(msg4_bytes) {
        Err(OwnOrPeerError::PeerError(s)) => {
            panic!("Received error msg: {}", s)
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            panic!("Send these bytes: {}", hexstring(&b))
        }
        Ok(val) => val,
    };
    return (ed_sck, ed_rck, rk_ed, as_sck, as_rck, as_rk, devaddr);
}

fn hexstring(slice: &[u8]) -> String {
    String::from("0x")
        + &slice
            .iter()
            .map(|n| format!("{:02X}", n))
            .collect::<Vec<String>>()
            .join(", 0x")
}
