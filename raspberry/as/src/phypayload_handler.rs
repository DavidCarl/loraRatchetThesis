/// This function removes the framecounter and the m type, and solely returns the message itself.
///     
/// # Arguments
///
/// * `msg` - the message which needs to be handled.
pub fn unpack_edhoc_first_message(msg: Vec<u8>) -> Vec<u8> {
    let msg = &msg[1..]; // fjerne mtype
    let _framecounter = &msg[0..2]; // gemme framecounter
    let msg = &msg[2..]; // fjerne frame counter
    msg.to_vec()
}

/// This function removes the framecounter and the m type, and returns the message and devaddr.
///     
/// # Arguments
///
/// * `msg` - the message which needs to be handled.
pub fn unpack_edhoc_message(msg: Vec<u8>) -> (Vec<u8>, [u8; 4]) {
    let msg = &msg[1..]; // fjerne mtype
    let msg = &msg[2..]; // fjerne frame counter
    let devaddr = msg[0..4].try_into().unwrap();
    let msg = &msg[4..];
    (msg.to_vec(), devaddr)
}
/// Pads the message we want to send with relevant data such as the mtype, devaddr and returns the message ready to send.
///     
/// # Arguments
///
/// * `msg` - The message you want to have padded with informatino
/// * `mtype` - The message type usually `0` or `2`
/// * `devaddr` - The dev addresse of the device
/// * `first_msg` - if its the first message being sent
pub fn prepare_message(msg: Vec<u8>, mtype: u8, devaddr: [u8; 4], first_msg: bool) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&mtype.to_be_bytes());
    if !first_msg {
        buffer.extend_from_slice(&devaddr);
    }
    buffer.extend_from_slice(&msg);
    buffer
}