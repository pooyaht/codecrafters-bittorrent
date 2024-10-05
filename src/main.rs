use std::{env, io::Read};

mod decoder;
mod encoder;
mod error;

use encoder::encode;

pub(crate) use error::*;
use sha1::Digest;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let mut bencode_decoder = decoder::Decoder::new(args[2].as_bytes());

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

        let mut bencode_decoder = decoder::Decoder::new(&buffer);
        let decoded_value = bencode_decoder.decode();
        if decoded_value.is_err() {
            panic!("{}", decoded_value.err().unwrap());
        }

        let decoded_value = decoded_value.unwrap();
        info_command(decoded_value);
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn info_command(decoded_value: serde_json::Value) {
    if let Some(announce) = decoded_value["announce"].as_str() {
        println!("Tracker URL: {}", announce);
    } else {
        println!("Tracker URL not found or not a string");
    }

    if let Some(length) = decoded_value["info"]["length"].as_i64() {
        println!("Length: {}", length);
    } else {
        println!("Length not found or not an integer");
    }

    let bencoded_info = encode(&decoded_value["info"]).unwrap_or_default();
    println!(
        "Info Hash: {}",
        format_args!("{:x}", sha1::Sha1::digest(bencoded_info))
    );
}
