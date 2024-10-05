use std::{env, io::Read};

mod decoder;
mod error;

pub(crate) use error::*;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let mut bencode_decoder = decoder::BenCodeDecoder::new(args[2].as_bytes());

        let decoded_value = bencode_decoder.decode();
        if decoded_value.is_err() {
            panic!("{}", decoded_value.err().unwrap());
        }
        println!("{}", decoded_value.unwrap());
    } else if command == "info" {
        let mut buffer = Vec::new();
        std::fs::File::open(args[2].as_str())
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();

        let mut bencode_decoder = decoder::BenCodeDecoder::new(&buffer);
        let decoded_value = bencode_decoder.decode();
        if decoded_value.is_err() {
            panic!("{}", decoded_value.err().unwrap());
        }
        let decoded_value = decoded_value.unwrap();

        if let Some(announce) = decoded_value["announce"].as_str() {
            println!("Tracker URL: {}", announce);
        } else {
            println!("Tracker URL not found or not a string");
        }

        if let Some(length) = decoded_value["info"]["length"].as_i64() {
            println!("Length: {} bytes", length);
        } else {
            println!("Length not found or not an integer");
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
