use std::{
    io::{Read, Write},
    net::SocketAddrV4,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Peer(pub SocketAddrV4);

impl Peer {
    pub(crate) fn handshake(
        &self,
        info_hash: &[u8; 20],
        peer_id: &[u8],
    ) -> Result<HandshakeResult, crate::Error> {
        let mut stream = std::net::TcpStream::connect(self.0)?;

        stream.write_all(&[19])?;
        stream.write_all(b"BitTorrent protocol")?;
        stream.write_all(&[0; 8])?;
        stream.write_all(info_hash)?;
        stream.write_all(peer_id)?;
        stream.flush()?;

        let mut buffer = [0; 20];
        stream.read_exact(&mut buffer)?;

        Ok(HandshakeResult { peer_id: buffer })
    }
}

pub struct HandshakeResult {
    pub peer_id: [u8; 20],
}
