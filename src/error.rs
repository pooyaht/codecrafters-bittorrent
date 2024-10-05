#[derive(Debug)]
pub(crate) enum Error {
    BencodeStringNoColon,
    BencodeStringLengthMismatch,
    NotNumber(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BencodeStringNoColon => write!(f, "bencode string has no colon"),
            Error::BencodeStringLengthMismatch => {
                write!(f, "bencode string length mismatch")
            }
            Error::NotNumber(number) => write!(f, "Not a number: {}", number),
        }
    }
}
