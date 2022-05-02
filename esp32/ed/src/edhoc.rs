use crate::hrng::HRNG;
use core::convert::TryInto;
use rand_core::RngCore;
use std::io::{Read, Write};
use std::net::TcpStream;

use oscore::edhoc::{
    error::{OwnError, OwnOrPeerError},
    util::build_error_message,
    PartyI,
};

use x25519_dalek_ng::{PublicKey, StaticSecret};

const DEVEUI: [u8; 8] = [0x1, 1, 2, 3, 2, 4, 5, 7];
const APPEUI: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

const SUITE_I: u8 = 3;
const METHOD_TYPE_I: u8 = 0;
const ED_KID: [u8; 1] = [0xA2];

const I_STATIC_MATERIAL: [u8; 32] = [
    154, 31, 220, 202, 59, 128, 114, 237, 96, 201, 18, 178, 29, 143, 85, 133, 70, 32, 155, 41, 124,
    111, 51, 127, 254, 98, 103, 99, 0, 38, 102, 4,
];

const R_STATIC_PK: [u8; 32] = [
    245, 156, 136, 87, 191, 59, 207, 135, 191, 100, 46, 213, 24, 152, 151, 45, 141, 35, 185, 103,
    168, 73, 74, 231, 37, 220, 227, 42, 68, 62, 196, 109,
];

pub struct EdhocMessage {
    pub m_type: u8,
    pub fcntup: [u8; 2],
    pub devaddr: [u8; 4],
    pub edhoc_msg: Vec<u8>,
}

pub fn join_procedure(stream: &mut TcpStream) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> {
    // The ED first creates keys, and generates initial state for sending
    let ed_static_priv = StaticSecret::from(I_STATIC_MATERIAL);
    let ed_static_pub = PublicKey::from(&ed_static_priv);

    let mut ed_ephemeral_keying = [0; 32];
    HRNG.fill_bytes(&mut ed_ephemeral_keying);

    let msg1_sender = PartyI::new(
        DEVEUI.to_vec(),
        Some(APPEUI.to_vec()),
        ed_ephemeral_keying,
        ed_static_priv,
        ed_static_pub,
        ED_KID.to_vec(),
    );

    let (msg1_bytes, msg2_receiver) = msg1_sender
        .generate_message_1(METHOD_TYPE_I, SUITE_I)
        .unwrap();

    let mut fcnt_up = 0;

    // The ED then prepares the first message into a appropriate phypayload, and send it
    let phypayload0 = prepare_edhoc_message(0, fcnt_up, None, msg1_bytes);
    fcnt_up += 1;
    stream.write(&phypayload0).expect("error during write");

    // The second message is now received from the AS, checked for mtype, and the phypayload fields are extracted
    let mut buf = [0; 128];
    let bytes_read = stream.read(&mut buf).expect("error during read");
    let phypayload1 = &buf[0..bytes_read];

    if phypayload1[0] != 1 {
        let err = build_error_message("bad mtype");
        stream.write(&err).expect("error during write");
        return None;
    }
    let msg2 = extract_edhoc_message(phypayload1)?;
    let devaddr = msg2.devaddr;

    // The ED extracts the kid from message 2
    let (_kid, appeui, msg2_verifier) =
        match msg2_receiver.unpack_message_2_return_kid(msg2.edhoc_msg) {
            Err(OwnOrPeerError::PeerError(s)) => {
                println!("received error {} in message 2, shutting down", s);
                return None;
            }
            Err(OwnOrPeerError::OwnError(b)) => {
                stream.write(&b).expect("error during write"); // in this case, return this errormessage
                return None;
            }
            Ok(val) => val,
        };

    if APPEUI.to_vec() != appeui {
        return None;
    }

    // With the kid, the ED can now retrieve the public static key and verify message 2
    let as_static_pub = PublicKey::from(R_STATIC_PK);
    let msg3_sender = match msg2_verifier.verify_message_2(as_static_pub.as_bytes()) {
        Err(OwnError(b)) => {
            stream.write(&b).expect("error during write");
            return None;
        }
        Ok(val) => val,
    };

    // now that the fields of message 2 has been fully verified, the ED can generate message 3
    let (msg4_receiver_verifier, msg3_bytes) = match msg3_sender.generate_message_3(None) {
        Err(OwnError(b)) => {
            stream.write(&b).expect("error during write");
            return None;
        }
        Ok(val) => val,
    };

    // Packing message 3 into a phypayload and sending it
    let phypayload2 = prepare_edhoc_message(2, fcnt_up, Some(devaddr), msg3_bytes);
    stream.write(&phypayload2).expect("error during write");

    // read message 4
    let mut buf = [0; 128];

    let bytes_read = stream.read(&mut buf).expect("error during read");
    let phypayload3 = &buf[0..bytes_read];

    if phypayload3[0] != 3 {
        let err = build_error_message("bad mtype");
        stream.write(&err).expect("error during write");
        return None;
    }
    let msg4 = extract_edhoc_message(phypayload3)?;
    let out = msg4_receiver_verifier.handle_message_4(msg4.edhoc_msg);

    let (ed_sck, ed_rck, ed_rk) = match out {
        Err(OwnOrPeerError::PeerError(s)) => {
            println!("received error {} in message 4, shutting down", s);
            return None;
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            stream.write(&b).expect("error during write");
            return None;
        }
        Ok(val) => val,
    };

    Some((ed_sck, ed_rck, ed_rk, devaddr.to_vec()))
}

fn prepare_edhoc_message(
    mtype: u8,
    fcnt: u16,
    devaddr: Option<[u8; 4]>,
    edhoc_msg: Vec<u8>,
) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::with_capacity(7 + edhoc_msg.len());
    buffer.extend_from_slice(&[mtype]);
    buffer.extend_from_slice(&fcnt.to_be_bytes());
    if devaddr != None {
        buffer.extend_from_slice(&devaddr.unwrap())
    };
    buffer.extend_from_slice(&edhoc_msg);

    buffer
}

fn extract_edhoc_message(msg: &[u8]) -> Option<EdhocMessage> {
    let m_type = msg[0];
    let fcntup = msg[1..3].try_into().ok()?;
    let devaddr = msg[3..7].try_into().ok()?;
    let edhoc_msg = msg[7..].try_into().ok()?;
    Some(EdhocMessage {
        m_type,
        fcntup,
        devaddr,
        edhoc_msg,
    })
}
