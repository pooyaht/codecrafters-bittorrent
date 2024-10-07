use clap::{Parser, Subcommand};
use std::{fs, io::Read, net::SocketAddrV4};

mod decoder;
mod encoder;
mod error;
mod peer;
mod torrent;
mod tracker;

pub(crate) use error::*;
use torrent::Torrent;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Decode {
        encoded_value: String,
    },
    Info {
        file_path: String,
    },
    Peers {
        file_path: String,
    },
    Handshake {
        torrent_file: String,
        peer_address: String,
    },
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Decode { encoded_value } => handle_decode_command(encoded_value),
        Commands::Info { file_path } => handle_info_command(file_path),
        Commands::Peers { file_path } => handle_peers_command(file_path),
        Commands::Handshake {
            torrent_file,
            peer_address,
        } => handle_handshake_command(torrent_file, peer_address),
    }
}

fn handle_decode_command(encoded_value: &str) -> Result<(), crate::Error> {
    let mut bencode_decoder = decoder::Decoder::new(encoded_value.as_bytes());
    let decoded_value = bencode_decoder.decode()?;
    println!("{}", decoded_value);
    Ok(())
}

fn handle_info_command(file_path: &str) -> Result<(), crate::Error> {
    let buffer = read_file(file_path)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    info_command(decoded_value);
    Ok(())
}

fn handle_peers_command(file_path: &str) -> Result<(), crate::Error> {
    let buffer = read_file(file_path)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    let torrent = Torrent::from_bencode(decoded_value)?;
    let tracker = tracker::Tracker::new(torrent.info.length as u64);
    let peers = tracker.get_peers(&torrent.announce, &url_encode(&torrent.info_hash()?))?;

    for peer in peers {
        println!("{}:{}", peer.0.ip(), peer.0.port());
    }
    Ok(())
}

fn handle_handshake_command(torrent_file: &str, peer_address: &str) -> Result<(), crate::Error> {
    let buffer = read_file(torrent_file)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    let torrent = Torrent::from_bencode(decoded_value)?;

    let peer_addr: SocketAddrV4 = peer_address.parse().expect("Invalid peer address");

    let info_hash = torrent.info_hash()?;
    let peer_id = "00112233445566778899".as_bytes();

    let peer = peer::Peer(peer_addr);
    let handshake_result = peer.handshake(&info_hash, peer_id)?;

    println!("Peer ID: {}", hex::encode(handshake_result.peer_id));

    Ok(())
}

fn read_file(path: &str) -> Result<Vec<u8>, crate::Error> {
    let mut buffer = Vec::new();
    let mut file = fs::File::open(path)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn info_command(decoded_value: serde_json::Value) {
    let torrent = Torrent::from_bencode(decoded_value).expect("Failed to parse torrent");
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
    let unreserved_characters =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    input
        .iter()
        .flat_map(|&byte| {
            if unreserved_characters.contains(&byte) {
                vec![byte as char]
            } else {
                format!("%{:02x}", byte).chars().collect()
            }
        })
        .collect()
}
