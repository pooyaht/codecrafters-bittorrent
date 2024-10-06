use std::{env, io::Read};

mod decoder;
mod encoder;
mod error;
mod torrent;
mod tracker;

pub(crate) use error::*;
use torrent::Torrent;

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
    } else if command == "peers" {
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
        let torrent = Torrent::from_bencode(decoded_value).unwrap();
        let tracker = tracker::Tracker::new(torrent.info.length as u64);
        let peers = tracker
            .get_peers(
                &torrent.announce,
                &url_encode(&torrent.info_hash().unwrap()),
            )
            .unwrap();

        for peer in peers {
            println!("{}:{}", peer.0.ip(), peer.0.port());
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn info_command(decoded_value: serde_json::Value) {
    let torrent = Torrent::from_bencode(decoded_value).unwrap();

    println!("Tracker URL: {}", torrent.announce);
    println!("Length: {}", torrent.info.length);
    println!(
        "Info Hash: {}",
        torrent
            .info_hash_hex_string()
            .expect("Failed to convert info hash to hex string")
    );
    println!("Piece Length: {}", torrent.info.piece_length);
    println!("Piece Hashes:");
    for hash in torrent.piece_hashes() {
        println!("{}", hash);
    }
}

fn url_encode(input: &[u8; 20]) -> String {
    let mut output = String::new();
    let unreserved_characters =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

    for &byte in input {
        if unreserved_characters.contains(byte as char) {
            output.push(byte as char);
        } else {
            output.push('%');
            output.push_str(&format!("{:02x}", byte));
        }
    }
    output
}
