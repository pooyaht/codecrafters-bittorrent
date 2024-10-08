use serde_json::Value;

use std::io::Write;

use crate::Error;

pub(crate) struct Encoder;

impl Encoder {
    pub(crate) fn encode(value: &Value) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        Self::encode_value(value, &mut output)?;
        Ok(output)
    }

    fn encode_value(value: &Value, output: &mut Vec<u8>) -> Result<(), Error> {
        match value {
            Value::String(s) => Self::encode_string(s.as_bytes(), output),
            Value::Number(n) => Self::encode_integer(n, output),
            Value::Array(a) => {
                if a.iter().all(|v| v.is_u64()) {
                    // Treat array of numbers as binary data
                    let bytes: Vec<u8> = a
                        .iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u8))
                        .collect();
                    Self::encode_string(&bytes, output)
                } else {
                    Self::encode_list(a, output)
                }
            }
            Value::Object(o) => Self::encode_dict(o, output),
            Value::Bool(b) => Self::encode_string(b.to_string().as_bytes(), output),
            Value::Null => Self::encode_string("null".as_bytes(), output),
        }
    }

    fn encode_string(s: &[u8], output: &mut Vec<u8>) -> Result<(), Error> {
        write!(output, "{}:", s.len())?;
        output.extend_from_slice(s);
        Ok(())
    }

    fn encode_integer(n: &serde_json::Number, output: &mut Vec<u8>) -> Result<(), Error> {
        write!(output, "i{}e", n)?;
        Ok(())
    }

    fn encode_list(arr: &[Value], output: &mut Vec<u8>) -> Result<(), Error> {
        output.push(b'l');
        for item in arr {
            Self::encode_value(item, output)?;
        }
        output.push(b'e');
        Ok(())
    }

    fn encode_dict(
        obj: &serde_json::Map<String, Value>,
        output: &mut Vec<u8>,
    ) -> Result<(), Error> {
        output.push(b'd');
        let mut keys: Vec<&String> = obj.keys().collect();
        keys.sort();
        for key in keys {
            Self::encode_string(key.as_bytes(), output)?;
            Self::encode_value(&obj[key], output)?;
        }
        output.push(b'e');
        Ok(())
    }
}
