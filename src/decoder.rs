use crate::Error;

pub(crate) struct BenCodeDecoder<'a> {
    pub(crate) input: &'a str,
}

impl<'a> BenCodeDecoder<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub(crate) fn decode(&self) -> Result<serde_json::Value, Error> {
        match self.input.chars().next() {
            Some(digit) if digit.is_ascii_digit() => self.parse_bencode_string(),
            Some('i') if self.input.ends_with('e') => self.parse_bencode_integer(),
            _ => todo!(),
        }
    }

    fn parse_bencode_string(&self) -> Result<serde_json::Value, crate::Error> {
        let encoded_value = self.input;

        let colon_index = encoded_value
            .find(':')
            .ok_or(Error::BencodeStringParseError)?;
        let number_string = &encoded_value[..colon_index];
        let number = number_string
            .parse::<i64>()
            .map_err(|_| Error::NotNumberError(number_string.to_string()))?;
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];

        Ok(serde_json::Value::String(string.to_string()))
    }

    fn parse_bencode_integer(&self) -> Result<serde_json::Value, Error> {
        let encoded_value = self.input;

        let number_string = &encoded_value[1..encoded_value.len() - 1];
        let number = number_string
            .parse::<i64>()
            .map_err(|_| Error::NotNumberError(number_string.to_string()))?;

        Ok(serde_json::Value::Number(number.into()))
    }
}
