use oscore::edhoc::{
    error::{OwnError, OwnOrPeerError},
    util::build_error_message,
    PartyR,
};

use std::net::{TcpStream};
use std::io::{Read, Write};

use rand_core::{OsRng,RngCore};


use x25519_dalek_ng::{PublicKey, StaticSecret};

const R_STATIC_MATERIAL: [u8; 32] = [
    59, 213, 202, 116, 72, 149, 45, 3, 163, 72, 11, 87, 152, 91, 221, 105, 241, 1, 101, 158, 72,
    69, 125, 110, 61, 244, 236, 138, 41, 140, 127, 132,
];
const I_STATIC_PK_MATERIAL: [u8; 32] = [
    205, 223, 6, 18, 99, 214, 239, 8, 65, 191, 174, 86, 128, 244, 122, 17, 32, 242, 101, 159, 17,
    91, 11, 40, 175, 120, 16, 114, 175, 213, 41, 47,
];

const DEVEUI: [u8; 8] = [0x1, 1, 2, 3, 2, 4, 5, 7];
const APPEUI: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
/// Runs the join procedure through the tcpstream with the ed
///
/// # Arguments
///
/// * `stream` - tcpstream connected to the as
/// 
/// # returns 
/// * (sending chain key, receiving chain key, root key, devaddr)
pub fn join_procedure(stream: &mut TcpStream) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> {
    let as_static_priv = StaticSecret::from(R_STATIC_MATERIAL);
    let as_static_pub = PublicKey::from(&as_static_priv);



    let mut as_ephemeral_keying = [0u8;32];
    OsRng.fill_bytes(&mut as_ephemeral_keying);

    let as_kid = [0xA3].to_vec();
    let msg1_receiver = PartyR::new(as_ephemeral_keying, as_static_priv, as_static_pub, as_kid);

    let mut buf = [0; 128];
    let bytes_read = stream.read(&mut buf).expect("stream reading error");

    let phypayload0 = &buf[0..bytes_read];

    if phypayload0[0] != 0 {
        let err = build_error_message("bad mtype");
        stream.write(&err).expect("stream writing error");
        return None;
    }
    let msg1 = unpack_edhoc_first_message(phypayload0);

    let mut fcnt_down = 0;
    let (msg2_sender, deveui, appeui) = match msg1_receiver.handle_message_1_ead(msg1.to_vec()) {
        Err(OwnError(b)) => {
            println!("sending error {:?}, ", b);
            stream.write(&b).expect("stream writing error");
            return None;
        }
        Ok(val) => val,
    };

    assert_eq!(APPEUI.to_vec(), appeui.unwrap());
    assert_eq!(DEVEUI.to_vec(), deveui);

    let (msg2_bytes, msg3_receiver) = match msg2_sender.generate_message_2(APPEUI.to_vec(), None) {
        Err(OwnOrPeerError::PeerError(s)) => {
            println!("received error {} generating message 2, shutting down", s);
            return None;
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            println!("sending error {:?}, ", b);
            stream.write(&b).expect("stream writing error");
            return None;
        }
        Ok(val) => val,
    };

    let mut devaddr = [0u8; 4];
    OsRng.fill_bytes(&mut devaddr);

    let phypayload1 = prepare_edhoc_message(1, fcnt_down, Some(devaddr), msg2_bytes);
    fcnt_down += 1;

    stream.write(&phypayload1).expect("stream writing error");

    let mut buf = [0; 128];
    let bytes_read = stream.read(&mut buf).expect("stream reading error");
    let phypayload2 = &buf[0..bytes_read];

    if phypayload2[0] != 2 {
        println!("receving bad mtype for message 3, closing connection...");
        let err = build_error_message("bad mtype");
        stream.write(&err).expect("stream writing error");
        return None;
    }

    let msg3 = unpack_edhoc_message(phypayload2)?;

    let (msg3verifier, _kid) = match msg3_receiver.unpack_message_3_return_kid(msg3.edhoc_msg) {
        Err(OwnOrPeerError::PeerError(s)) => {
            println!("received error {} in message 3, shutting down", s);
            return None;
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            println!("sending error {:?}, ", b);
            stream.write(&b).expect("stream writing error");
            return None;
        }
        Ok(val) => val,
    };

    let ed_static_pub = PublicKey::from(I_STATIC_PK_MATERIAL);

    let (msg4_sender, as_sck, as_rck, as_rk) =
        match msg3verifier.verify_message_3(ed_static_pub.as_bytes()) {
            Err(OwnOrPeerError::PeerError(s)) => {
                println!(
                    "received error {} while verifying message 3, shutting down",
                    s
                );
                return None;
            }
            Err(OwnOrPeerError::OwnError(b)) => {
                println!("sending error {:?}, ", b);
                stream.write(&b).expect("stream writing error");
                return None;
            }
            Ok(val) => val,
        };

    let msg4_bytes = match msg4_sender.generate_message_4(None) {
        Err(OwnOrPeerError::PeerError(s)) => {
            println!(
                "received error {} while generating message 4, shutting down",
                s
            );
            return None;
        }
        Err(OwnOrPeerError::OwnError(b)) => {
            println!("sending error {:?}, ", b);
            stream.write(&b).expect("stream writing error"); // in this case, return this errormessage
            return None;
        }

        Ok(val) => val,
    };
    let phypayload3 = prepare_edhoc_message(3, fcnt_down, Some(devaddr), msg4_bytes);
    stream.write(&phypayload3).expect("stream writing error");
    return Some((as_sck, as_rck, as_rk, devaddr.to_vec()));
}

struct EdhocMessage {
    _m_type: u8,
    _fcntup: [u8; 2],
    _devaddr: [u8; 4],
    edhoc_msg: Vec<u8>,
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

fn unpack_edhoc_message(msg: &[u8]) -> Option<EdhocMessage> {
    let m_type = msg[0];
    let fcntup = msg[1..3].try_into().ok()?;
    let devaddr = msg[3..7].try_into().ok()?;
    let edhoc_msg = msg[7..].try_into().ok()?;
    Some(EdhocMessage {
        _m_type: m_type,
        _fcntup: fcntup,
        _devaddr: devaddr,
        edhoc_msg,
    })
}

fn unpack_edhoc_first_message(msg: &[u8]) -> Vec<u8> {
    let msg = &msg[1..];
    let _framecounter = &msg[0..2];
    let msg = &msg[2..]; 
    msg.to_vec()
}
