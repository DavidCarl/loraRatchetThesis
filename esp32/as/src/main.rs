use oscore::edhoc::{
    error::{OwnError, OwnOrPeerError},
    util::build_error_message,
    PartyR,
};
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use twoRatchet::AS::ASRatchet;

use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_core::OsRng;
use x25519_dalek_ng::{PublicKey, StaticSecret};

const R_STATIC_MATERIAL: [u8; 32] = [
    59, 213, 202, 116, 72, 149, 45, 3, 163, 72, 11, 87, 152, 91, 221, 105, 241, 1, 101, 158, 72,
    69, 125, 110, 61, 244, 236, 138, 41, 140, 127, 132,
];
const I_STATIC_PK_MATERIAL: [u8; 32] = [
    205, 223, 6, 18, 99, 214, 239, 8, 65, 191, 174, 86, 128, 244, 122, 17, 32, 242, 101, 159, 17,
    91, 11, 40, 175, 120, 16, 114, 175, 213, 41, 47,
];

mod edhoc;

const DEVEUI: [u8; 8] = [0x1, 1, 2, 3, 2, 4, 5, 7];
const APPEUI: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("192.168.1.227:8888").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        handle_connection(&mut stream)?;
    }

    Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), Error> {
    println!("incoming connection from: {}", stream.peer_addr()?);

    // Running EDHOC join procedure
    let (as_sck, as_rck, as_rk, devaddr) = match edhoc::join_procedure(stream) {
        Some(join_output) => join_output,
        None => return Ok(()),
    };

    let mut ratchet = ASRatchet::new(
        as_rk.try_into().unwrap(),
        as_rck.try_into().unwrap(),
        as_sck.try_into().unwrap(),
        devaddr.try_into().unwrap(),
        OsRng,
    );

    let mut n = 0;
    loop {
        let mut buf = [0; 64];
        stream.read_exact(&mut buf)?;
        let incoming = &buf;
        println!("getting {:?}", incoming);
        let (newout, sendnew) = match ratchet.receive(incoming.to_vec()) {
            Ok((x, b)) => (x, b),
            Err(e) => {
                println!("error has happened {:?}", incoming);
                continue;
            }
        };

        if !sendnew {
        } else {
            match stream.write(&newout) {
                Ok(_) => println!("ok"),
                Err(x) => println!("err {:?}", x),
            }
        }
        n += 1;
        println!("n {}", n);
    }
}





