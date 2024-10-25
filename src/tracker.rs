use std::net::SocketAddrV4;

use serde::Serialize;

use crate::decoder;

#[derive(Serialize)]
pub(crate) struct Tracker {
    peer_id: String,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    compact: u8,
}

impl Tracker {
    pub(crate) fn new(left: u64) -> Self {
        Self {
            peer_id: "00112233445566778899".to_owned(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left,
            compact: 1,
        }
    }

    pub(crate) fn get_peers(
        &self,
        announce_url: &str,
        info_hash: &str,
    ) -> Result<Vec<SocketAddrV4>, crate::Error> {
        let url = format!(
            "{}?info_hash={}&{}",
            announce_url,
            info_hash,
            serde_urlencoded::to_string(self).expect("Tracker is not url encodable")
        );

        let response = reqwest::blocking::get(url)?;
        let bytes = response.bytes().unwrap();
        let mut decoder = decoder::Decoder::new(&bytes);
        let decoded_value = decoder.decode().unwrap();

        decoded_value
            .as_object()
            .map(|obj| {
                obj["peers"].as_array().map(|arr| {
                    arr.chunks(6)
                        .map(|chunk| {
                            let ip_bytes: [u8; 4] = [
                                chunk[0].as_u64().unwrap_or_default() as u8,
                                chunk[1].as_u64().unwrap_or_default() as u8,
                                chunk[2].as_u64().unwrap_or_default() as u8,
                                chunk[3].as_u64().unwrap_or_default() as u8,
                            ];
                            let ip = std::net::Ipv4Addr::from(ip_bytes);
                            let port = u16::from_be_bytes([
                                chunk[4].as_u64().unwrap_or_default() as u8,
                                chunk[5].as_u64().unwrap_or_default() as u8,
                            ]);

                            SocketAddrV4::new(ip, port)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .unwrap_or_default()
            .ok_or(crate::Error::NoPeers)
    }
}
