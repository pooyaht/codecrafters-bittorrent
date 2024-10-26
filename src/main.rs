use clap::{Parser, Subcommand};
use peer::{PeerConnection, PeerMessageType, PiecePayload, RequestPayload};
use sha1::Digest;
use std::{
    fs,
    io::{Read, Write},
    net::SocketAddrV4,
    path::PathBuf,
};

mod decoder;
mod encoder;
mod error;
mod handshake;
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
        file_path: PathBuf,
    },
    Peers {
        file_path: PathBuf,
    },
    Handshake {
        torrent_file: PathBuf,
        peer_address: String,
    },
    #[command(name = "download_piece")]
    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        piece: usize,
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
        Commands::DownloadPiece {
            output,
            torrent,
            piece,
        } => handle_download_piece_command(output, torrent, *piece),
    }
}

fn handle_decode_command(encoded_value: &str) -> Result<(), crate::Error> {
    let mut bencode_decoder = decoder::Decoder::new(encoded_value.as_bytes());
    let decoded_value = bencode_decoder.decode()?;
    println!("{}", decoded_value);
    Ok(())
}

fn handle_info_command(file_path: &PathBuf) -> Result<(), crate::Error> {
    let buffer = read_file(file_path)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    info_command(decoded_value);
    Ok(())
}

fn handle_peers_command(file_path: &PathBuf) -> Result<(), crate::Error> {
    let buffer = read_file(file_path)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    let torrent = Torrent::from_bencode(decoded_value)?;
    let tracker = tracker::Tracker::new(torrent.info.length as u64);
    let peers = tracker.get_peers(&torrent.announce, &url_encode(&torrent.info_hash()?))?;

    for peer in peers {
        println!("{}:{}", peer.ip(), peer.port());
    }
    Ok(())
}

fn handle_handshake_command(
    torrent_file: &PathBuf,
    peer_address: &str,
) -> Result<(), crate::Error> {
    let buffer = read_file(torrent_file)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    let torrent = Torrent::from_bencode(decoded_value)?;

    let peer_addr: SocketAddrV4 = peer_address.parse().expect("Invalid peer address");

    let info_hash = torrent.info_hash()?;
    let peer_id = b"00112233445566778899".to_owned();

    let mut handshake = handshake::Handshake::new(info_hash, peer_id);
    let handshake_bytes = handshake.as_bytes_mut();

    let mut peer = std::net::TcpStream::connect(peer_addr)?;

    peer.write_all(handshake_bytes).unwrap();
    peer.read_exact(handshake_bytes).unwrap();

    println!("Peer ID: {}", hex::encode(handshake.peer_id));

    Ok(())
}

fn handle_download_piece_command(
    output: &PathBuf,
    torrent: &PathBuf,
    piece_index: usize,
) -> Result<(), crate::Error> {
    let buffer = read_file(torrent)?;
    let mut bencode_decoder = decoder::Decoder::new(&buffer);
    let decoded_value = bencode_decoder.decode()?;
    let torrent = Torrent::from_bencode(decoded_value)?;
    let tracker = tracker::Tracker::new(torrent.info.length as u64);
    let peers = tracker.get_peers(&torrent.announce, &url_encode(&torrent.info_hash()?))?;
    let info_hash = torrent.info_hash()?;
    let peer_id = b"00112233445566778899".to_owned();
    let mut handshake = handshake::Handshake::new(info_hash, peer_id);
    let handshake_bytes = handshake.as_bytes_mut();

    let peer_addr: SocketAddrV4 = peers[0];
    let mut peer = std::net::TcpStream::connect(peer_addr)?;

    peer.write_all(handshake_bytes)?;
    peer.read_exact(handshake_bytes)?;

    let mut peer_connection = PeerConnection::new(peer);
    peer_connection.expect_message(PeerMessageType::Bitfield)?;

    peer_connection.send_message(PeerMessageType::Interested, &[])?;
    peer_connection.expect_message(PeerMessageType::Unchoke)?;

    let num_pieces = torrent.num_pieces();
    let piece_size = if piece_index == (num_pieces - 1) as usize {
        if torrent.info.length % torrent.info.piece_length == 0 {
            torrent.info.piece_length
        } else {
            torrent.info.length % torrent.info.piece_length
        }
    } else {
        torrent.info.piece_length
    };

    const BLOCK_SIZE: u32 = 2u32.pow(14);

    let mut piece = vec![0u8; piece_size as usize];

    for (i, chunk) in piece.chunks_mut(BLOCK_SIZE as usize).enumerate() {
        let request_payload = RequestPayload::new(
            piece_index as u32,
            i as u32 * BLOCK_SIZE,
            chunk.len() as u32,
        );

        peer_connection.send_message(PeerMessageType::Request, &request_payload.as_bytes())?;
        let peer_message = peer_connection.expect_message(PeerMessageType::Piece)?;

        let piece_payload = PiecePayload::from(peer_message.payload.as_slice());
        chunk.copy_from_slice(&piece_payload.block);
    }

    let piece_hashes = torrent.piece_hashes();
    let piece_hash = &piece_hashes[piece_index];
    assert_eq!(*piece_hash, hex::encode(sha1::Sha1::digest(&piece)));

    let mut file = fs::File::create(output)?;
    file.write_all(&piece)?;

    Ok(())
}

fn read_file(path: &PathBuf) -> Result<Vec<u8>, crate::Error> {
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
