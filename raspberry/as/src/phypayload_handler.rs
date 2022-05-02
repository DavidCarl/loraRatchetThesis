static mut FCNTDOWN: u16 = 0;


pub struct EdhocPhypayload {
    pub _m: u8,
    pub _fcntup: [u8; 2],
    pub devaddr: [u8; 4],
    pub msg: Vec<u8>,
}

/// Here we seperate the function into multiple smaller information bites, so its more usefull.
///     
/// # Arguments
///
/// * `ogmsg` - the message which needs to be handled.
/// * `first` - Is it the first message, in that case, no devaddr should be appended
pub fn remove_message(ogmsg: Vec<u8>, first: bool) -> EdhocPhypayload {
    if first {
        EdhocPhypayload {
            _m: ogmsg[0],
            _fcntup: ogmsg[1..3].try_into().unwrap(),
            devaddr: [0,0,0,0],
            msg: ogmsg[3..].try_into().unwrap()
        }
    }else{
        EdhocPhypayload {
            _m: ogmsg[0],
            _fcntup: ogmsg[1..3].try_into().unwrap(),
            devaddr: ogmsg[3..7].try_into().unwrap(),
            msg: ogmsg[7..].try_into().unwrap(),
        }
    }
}

/// Pads the message we want to send with relevant data such as the mtype, devaddr and returns the message ready to send.
///     
/// # Arguments
///
/// * `msg` - The message you want to have padded with informatino
/// * `mtype` - The message type usually `0` or `2`
/// * `devaddr` - The dev addresse of the device
/// * `first_msg` - if its the first message being sent
pub fn prepare_message(msg: Vec<u8>, mtype: u8, devaddr: Option<[u8; 4]>) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&mtype.to_be_bytes());
    unsafe {
        buffer.extend_from_slice(&FCNTDOWN.to_be_bytes());
        FCNTDOWN += 1;
    }
    match devaddr {
        Some(addr) => buffer.extend_from_slice(&addr),
        None => ()
    }
    buffer.extend_from_slice(&msg);
    buffer
}