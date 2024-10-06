use std::net::SocketAddrV4;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Peer(pub SocketAddrV4);
