use crate::Error;

pub(crate) struct BenCodeDecoder<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> BenCodeDecoder<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    pub(crate) fn decode(&mut self) -> Result<serde_json::Value, Error> {
        let encoded_value = &self.input[self.index..];
        match encoded_value.chars().next() {
            Some(digit) if digit.is_ascii_digit() => self.parse_bencode_string(),
            Some('i') => self.parse_bencode_integer(),
            Some('l') => self.parse_bencode_list(),
            Some('d') => self.parse_bencode_dict(),
            _ => todo!(),
        }
    }

    fn parse_bencode_string(&mut self) -> Result<serde_json::Value, crate::Error> {
        let encoded_value = &self.input[self.index..];
        let colon_index = encoded_value.find(':').ok_or(Error::BencodeStringNoColon)?;
        let number_string = &encoded_value[..colon_index];
        let number = number_string
            .parse::<i64>()
            .map_err(|_| Error::NotNumber(number_string.to_string()))?;

        if colon_index + number as usize >= encoded_value.len() {
            return Err(Error::BencodeStringLengthMismatch);
        }

        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        self.index += number as usize + 1 + colon_index;

        Ok(serde_json::Value::String(string.to_string()))
    }

    fn parse_bencode_integer(&mut self) -> Result<serde_json::Value, Error> {
        let encoded_value = &self.input[self.index..].split('e').next().unwrap()[1..];
        let number = encoded_value
            .parse::<i64>()
            .map_err(|_| Error::NotNumber(encoded_value.to_string()))?;

        // Skip the 'i' and the 'e'
        self.index += encoded_value.len() + 1 + 1;

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
        while !encoded_value.starts_with('e') {
            list.push(self.decode()?);
            encoded_value = &self.input[self.index..];
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
        while !encoded_value.starts_with('e') {
            let key = self.decode()?;
            let value = self.decode()?;
            let key = key.as_str().ok_or(Error::InvalidDictKey(key.to_string()))?;
            dict.insert(key.to_string(), value);
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
        let mut bencode_decoder = BenCodeDecoder::new("d3:foo3:bar5:helloi52ee");
        let decoded_value = bencode_decoder.decode();
        assert!(decoded_value.is_ok());
        assert_eq!(
            decoded_value.unwrap(),
            serde_json::Value::Object(serde_json::Map::from_iter(vec![
                (
                    "foo".to_string(),
                    serde_json::Value::String("bar".to_string())
                ),
                ("hello".to_string(), serde_json::Value::Number(52.into()))
            ]))
        );
    }
}
