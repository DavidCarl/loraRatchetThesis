extern crate linux_embedded_hal as hal;
extern crate sx127x_lora;

mod filehandling;
mod edhoc;
mod lora_handler;
mod ratchet;
mod phypayload_handler;

fn main() {
    let config: filehandling::Config = filehandling::load_config("./config.json".to_string());
    let enc_keys: filehandling::StaticKeys = filehandling::load_static_keys("./keys.json".to_string());
    let lora = &mut lora_handler::setup_sx127x(250000, 7);
    let rtn = edhoc::handshake(lora, enc_keys, config.deveui, config.appeui, config).unwrap();
    ratchet::run(lora, rtn, config.dhr_const, config);
}

