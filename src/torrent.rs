use serde_json::Value;
use sha1::Digest;

pub struct Torrent {
    pub announce: String,
    pub info: TorrentInfo,
}

pub struct TorrentInfo {
    pub length: i64,
    pub name: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

impl Torrent {
    pub fn from_bencode(value: Value) -> Result<Self, crate::Error> {
        let announce = value["announce"]
            .as_str()
            .ok_or(crate::Error::MissingField("announce".to_owned()))?
            .to_string();

        let info = value["info"]
            .as_object()
            .ok_or(crate::Error::MissingField("info".to_owned()))?;

        let length = info["length"]
            .as_i64()
            .ok_or(crate::Error::MissingField("length".to_owned()))?;

        let name = info["name"]
            .as_str()
            .ok_or(crate::Error::MissingField("name".to_owned()))?
            .to_string();

        let piece_length = info["piece length"]
            .as_i64()
            .ok_or(crate::Error::MissingField("piece length".to_owned()))?;

        let pieces = info["pieces"]
            .as_array()
            .ok_or(crate::Error::MissingField("pieces".to_owned()))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();

        Ok(Torrent {
            announce,
            info: TorrentInfo {
                length,
                name,
                piece_length,
                pieces,
            },
        })
    }

    pub fn info_hash(&self) -> Result<[u8; 20], crate::Error> {
        let encoded_info =
            crate::encoder::Encoder::encode(&Value::Object(serde_json::Map::from_iter(vec![
                ("length".to_string(), Value::Number(self.info.length.into())),
                ("name".to_string(), Value::String(self.info.name.clone())),
                (
                    "piece length".to_string(),
                    Value::Number(self.info.piece_length.into()),
                ),
                (
                    "pieces".to_string(),
                    Value::Array(
                        self.info
                            .pieces
                            .iter()
                            .map(|&b| Value::Number(b.into()))
                            .collect(),
                    ),
                ),
            ])))?;

        Ok(sha1::Sha1::digest(&encoded_info).into())
    }

    pub fn piece_hashes(&self) -> Vec<String> {
        self.info
            .pieces
            .chunks(20)
            .fold(Vec::new(), |mut acc, chunk| {
                let mut buffer = String::with_capacity(40);
                for piece in chunk {
                    buffer.push_str(&format!("{:02x}", piece));
                }
                acc.push(buffer);
                acc
            })
    }
}
