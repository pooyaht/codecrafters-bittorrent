#[derive(Debug)]
pub(crate) enum Error {
    BencodeStringParseError,
    NotNumberError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BencodeStringParseError => write!(f, "bencode string parse error"),
            Error::NotNumberError(number) => write!(f, "not a number: {}", number),
        }
    }
}
