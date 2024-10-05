use serde_json::json;

use crate::Error;

pub(crate) struct BenCodeDecoder<'a> {
    input: &'a [u8],
    index: usize,
}

impl<'a> BenCodeDecoder<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self { input, index: 0 }
    }

    pub(crate) fn decode(&mut self) -> Result<serde_json::Value, Error> {
        let encoded_value = &self.input[self.index..];
        match encoded_value.first() {
            Some(digit) if digit.is_ascii_digit() => self.parse_bencode_string(),
            Some(b'i') => self.parse_bencode_integer(),
            Some(b'l') => self.parse_bencode_list(),
            Some(b'd') => self.parse_bencode_dict(),
            Some(val) => Err(Error::InvalidBencodeType(*val)),
            None => Err(Error::IsEmpty),
        }
    }

    fn parse_bencode_string(&mut self) -> Result<serde_json::Value, crate::Error> {
        let encoded_value = &self.input[self.index..];

        let colon_index = encoded_value
            .iter()
            .position(|&x| x == b':')
            .ok_or(Error::BencodeStringNoColon)?;
        let number_string =
            std::str::from_utf8(&encoded_value[..colon_index]).map_err(|_| Error::InvalidUTF8)?;
        let number = number_string
            .parse::<i64>()
            .map_err(|_| Error::NotNumber(number_string.to_string()))?;

        if colon_index + number as usize >= encoded_value.len() {
            return Err(Error::BencodeStringLengthMismatch);
        }

        let bytes = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        self.index += number as usize + 1 + colon_index;

        //TODO handle non-UTF8 strings better
        match std::str::from_utf8(bytes) {
            Ok(s) => Ok(serde_json::Value::String(s.to_string())),
            _ => Ok(json!(bytes.to_vec())),
        }
    }

    fn parse_bencode_integer(&mut self) -> Result<serde_json::Value, Error> {
        let encoded_value = &self.input[self.index + 1..];

        let end_index = encoded_value
            .iter()
            .position(|&x| x == b'e')
            .ok_or(Error::MissingTerminator)?;
        let number_str =
            std::str::from_utf8(&encoded_value[..end_index]).map_err(|_| Error::InvalidUTF8)?;
        let number = number_str
            .parse::<i64>()
            .map_err(|_| Error::NotNumber(number_str.to_string()))?;

        self.index += end_index + 2; // Skip 'i', number, and 'e'

        Ok(serde_json::Value::Number(number.into()))
    }

    fn parse_bencode_list(&mut self) -> Result<serde_json::Value, Error> {
        // Skip the 'l'
        self.index += 1;
        let array = self.parse_bencode_list_inner()?;
        // Skip the 'e'
        self.index += 1;

        Ok(serde_json::Value::Array(array))
    }

    fn parse_bencode_list_inner(&mut self) -> Result<Vec<serde_json::Value>, Error> {
        let mut list = Vec::new();

        let mut encoded_value = &self.input[self.index..];

        while self.index < self.input.len() && encoded_value[0] != b'e' {
            list.push(self.decode()?);
            encoded_value = &self.input[self.index..];
        }

        if self.index >= self.input.len() {
            return Err(Error::UnexpectedEOF);
        }

        Ok(list)
    }

    fn parse_bencode_dict(&mut self) -> Result<serde_json::Value, Error> {
        // Skip the 'd'
        self.index += 1;
        let dict = self.parse_bencode_dict_inner()?;
        // Skip the 'e'
        self.index += 1;

        Ok(serde_json::Value::Object(dict))
    }

    fn parse_bencode_dict_inner(
        &mut self,
    ) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
        let mut dict = serde_json::Map::new();

        let mut encoded_value = &self.input[self.index..];
        while self.index < self.input.len() && encoded_value[0] != b'e' {
            let key = self.decode()?;
            let value = self.decode()?;
            let key = match key {
                serde_json::Value::String(s) => s,
                _ => return Err(Error::InvalidDictKey(format!("{:?}", key))),
            };

            if self.index >= self.input.len() {
                return Err(Error::UnexpectedEOF);
            }

            dict.insert(key, value);
            encoded_value = &self.input[self.index..];
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bencode_dict_decoder() {
        let input = b"d3:foo3:bar5:helloi52ee";
        let mut bencode_decoder = BenCodeDecoder::new(input);
        let decoded_value = bencode_decoder.decode();
        assert!(decoded_value.is_ok());
        assert_eq!(
            decoded_value.unwrap(),
            serde_json::json!({
                "foo": "bar",
                "hello": 52
            })
        );
    }

    #[test]
    fn test_bencode_list_decoder() {
        let input = b"l3:foo3:bari52ee";
        let mut bencode_decoder = BenCodeDecoder::new(input);
        let decoded_value = bencode_decoder.decode();
        assert!(decoded_value.is_ok());
        assert_eq!(
            decoded_value.unwrap(),
            serde_json::json!(["foo", "bar", 52])
        );
    }

    #[test]
    fn test_bencode_integer_decoder() {
        let input = b"i-52e";
        let mut bencode_decoder = BenCodeDecoder::new(input);
        let decoded_value = bencode_decoder.decode();
        assert!(decoded_value.is_ok());
        assert_eq!(decoded_value.unwrap(), serde_json::json!(-52));
    }

    #[test]
    fn test_bencode_string_decoder() {
        let input = b"5:hello";
        let mut bencode_decoder = BenCodeDecoder::new(input);
        let decoded_value = bencode_decoder.decode();
        assert!(decoded_value.is_ok());
        assert_eq!(decoded_value.unwrap(), serde_json::json!("hello"));
    }

    #[test]
    fn test_invalid_bencode() {
        let input = b"x:invalid";
        let mut bencode_decoder = BenCodeDecoder::new(input);
        let decoded_value = bencode_decoder.decode();
        assert!(decoded_value.is_err());
        assert!(matches!(
            decoded_value.unwrap_err(),
            Error::InvalidBencodeType(b'x')
        ));
    }
}
