#[derive(Debug)]
pub(crate) enum Error {
    BencodeStringNoColon,
    BencodeStringLengthMismatch,
    NotNumber(String),
    InvalidDictKey(String),
    InvalidBencodeType(u8),
    IsEmpty,
    InvalidUTF8,
    MissingTerminator,
    UnexpectedEOF,
    Io(std::io::Error),
    Network(reqwest::Error),
    NoPeers,
    MissingField(String),
    InvalidMessageType(u8),
    UnexpectedPeerMessage(u8, u8),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BencodeStringNoColon => write!(f, "bencode string has no colon"),
            Error::BencodeStringLengthMismatch => {
                write!(f, "bencode string length mismatch")
            }
            Error::NotNumber(number) => write!(f, "Not a number: {}", number),
            Error::InvalidDictKey(key) => write!(f, "Invalid dict key: {}", key),
            Error::InvalidBencodeType(bencode_type) => {
                write!(f, "Invalid bencode type: {}", bencode_type)
            }
            Error::IsEmpty => write!(f, "Input is empty"),
            Error::InvalidUTF8 => write!(f, "Invalid UTF-8"),
            Error::MissingTerminator => write!(f, "Missing terminator"),
            Error::UnexpectedEOF => write!(f, "Unexpected EOF"),
            Error::Io(e) => write!(f, "File not found: {}", e),
            Error::MissingField(field) => write!(f, "Missing field: {}", field),
            Error::Network(e) => write!(f, "Network error: {}", e),
            Error::NoPeers => write!(f, "Couldn't find any peers"),
            Error::InvalidMessageType(message_type) => {
                write!(f, "Invalid message type: {}", message_type)
            }
            Error::UnexpectedPeerMessage(expected, actual) => {
                write!(
                    f,
                    "Unexpected peer message: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Network(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Network(value)
    }
}
