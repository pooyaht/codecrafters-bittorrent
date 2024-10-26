use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PeerMessageType {
    Unchoke = 1,
    Interested = 2,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct PeerMessage {
    pub(crate) length: u32,
    pub(crate) id: PeerMessageType,
    pub(crate) payload: Vec<u8>,
}

#[repr(C)]
pub(crate) struct RequestPayload {
    pub(crate) index: u32,
    pub(crate) begin: u32,
    pub(crate) length: u32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct PiecePayload {
    pub(crate) index: u32,
    pub(crate) begin: u32,
    pub(crate) block: Vec<u8>,
}

impl From<&[u8]> for PiecePayload {
    fn from(bytes: &[u8]) -> Self {
        let index = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let begin = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let block = bytes[8..].to_vec();
        Self {
            index,
            begin,
            block,
        }
    }
}

impl RequestPayload {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            index,
            begin,
            length,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.index.to_be_bytes());
        buffer.extend_from_slice(&self.begin.to_be_bytes());
        buffer.extend_from_slice(&self.length.to_be_bytes());
        buffer
    }
}

pub struct PeerConnection {
    stream: TcpStream,
}

impl PeerConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn send_message(
        &mut self,
        message_type: PeerMessageType,
        payload: &[u8],
    ) -> Result<(), Error> {
        let length = 1 + payload.len() as u32;

        self.stream.write_all(&length.to_be_bytes())?;

        let message_id = message_type as u8;
        self.stream.write_all(&[message_id])?;

        if !payload.is_empty() {
            self.stream.write_all(payload)?;
        }

        Ok(())
    }

    pub fn read_message(&mut self) -> Result<PeerMessage, Error> {
        let mut length_buf = [0u8; 4];
        self.stream.read_exact(&mut length_buf)?;
        let length = u32::from_be_bytes(length_buf);

        let mut message_buf = vec![0u8; length as usize];
        self.stream.read_exact(&mut message_buf)?;

        let message_type = match message_buf[0] {
            1 => PeerMessageType::Unchoke,
            5 => PeerMessageType::Bitfield,
            6 => PeerMessageType::Request,
            7 => PeerMessageType::Piece,
            id => return Err(Error::InvalidMessageType(id)),
        };

        let payload = if message_buf.len() > 1 {
            message_buf[1..].to_vec()
        } else {
            Vec::new()
        };

        Ok(PeerMessage {
            length,
            id: message_type,
            payload,
        })
    }

    pub fn expect_message(&mut self, expected_type: PeerMessageType) -> Result<PeerMessage, Error> {
        let message = self.read_message()?;
        if message.id != expected_type {
            return Err(Error::UnexpectedPeerMessage(
                expected_type as u8,
                message.id as u8,
            ));
        }
        Ok(message)
    }
}
