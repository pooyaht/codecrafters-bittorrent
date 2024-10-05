use std::env;

mod decoder;
mod error;

pub(crate) use error::*;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let mut bencode_decoder = decoder::BenCodeDecoder::new(&args[2]);

        let decoded_value = bencode_decoder.decode();
        if decoded_value.is_err() {
            panic!("{}", decoded_value.err().unwrap());
        }
        println!("{}", decoded_value.unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
