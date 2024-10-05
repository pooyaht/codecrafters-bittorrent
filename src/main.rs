use std::env;

mod error;

use anyhow::Result;
pub(crate) use error::*;

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value, Error> {
    // If encoded_value starts with a digit, it's a number
    if encoded_value.chars().next().unwrap().is_ascii_digit() {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value
            .find(':')
            .ok_or(Error::BencodeStringParseError)?;
        let number_string = &encoded_value[..colon_index];
        let number = number_string
            .parse::<i64>()
            .map_err(|_| Error::NotNumberError(number_string.to_string()))?;
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        Ok(serde_json::Value::String(string.to_string()))
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        if decoded_value.is_err() {
            println!("{}", decoded_value.err().unwrap());
            return;
        }
        println!("{}", decoded_value.unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
