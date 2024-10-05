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
        }
    }
}
