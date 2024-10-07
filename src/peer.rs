use std::{
    fmt,
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
        peer_id: &[u8; 20],
    ) -> Result<HandshakeResult, crate::Error> {
        let mut stream = std::net::TcpStream::connect(self.0)?;

        stream.write_all(&[19])?;
        stream.write_all(b"BitTorrent protocol")?;
        stream.write_all(&[0; 8])?;
        stream.write_all(info_hash)?;
        stream.write_all(peer_id)?;
        stream.flush()?;

        let mut buffer = [0; 68];
        stream.read_exact(&mut buffer)?;

        let mut peer_id = [0_u8; 20];

        peer_id.iter_mut().enumerate().for_each(|(i, byte)| {
            *byte = buffer[48 + i];
        });

        Ok(HandshakeResult { peer_id })
    }
}

pub struct HandshakeResult {
    pub peer_id: [u8; 20],
}

impl fmt::Display for HandshakeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Peer ID: ")?;
        for byte in &self.peer_id {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
