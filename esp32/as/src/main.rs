use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use twoRatchet::AS::ASRatchet;

use rand_core::OsRng;

mod edhoc;

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
            Err(_e) => {
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





