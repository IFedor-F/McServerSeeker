use super::{McReadBuf, McVarInt, McVarIntError};
use bytes::{Buf, Bytes};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MCJsonTextFieldError {
    #[error("McJsonText error: size ({actual} bytes) is more than limit ({max} bytes)")]
    TooLong { actual: usize, max: usize },

    #[error("McJsonText error: length of JSON string ({0}) can't be less than zero")]
    InvalidLength(i32),

    #[error("McJsonText error: Not enough bytes in the buffer to read JSON")]
    UnexpectedEof,

    #[error("McVarInt error while reading McJsonText: {0}")]
    InvalidVarInt(#[from] McVarIntError),

    #[error("JSON parsing error while reading McJsonText: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct McJsonTextComponent(serde_json::Value);

impl McJsonTextComponent {
    fn extract_raw_text(value: &serde_json::Value, out: &mut String) {
        let mut stack = vec![value];

        while let Some(current) = stack.pop() {
            match current {
                serde_json::Value::String(s) => {
                    out.push_str(s);
                }
                serde_json::Value::Array(arr) => {
                    for item in arr.iter().rev() {
                        stack.push(item);
                    }
                }
                serde_json::Value::Object(obj) => {
                    if let Some(extra) = obj.get("extra") {
                        stack.push(extra);
                    }
                    if let Some(serde_json::Value::String(text)) = obj.get("text") {
                        out.push_str(text);
                    }
                    if let Some(serde_json::Value::String(translate_key)) = obj.get("translate") {
                        out.push_str(translate_key);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn formatted(&self) -> String {
        let mut result = String::new();
        Self::extract_raw_text(&self.0, &mut result);
        result
    }
}

#[derive(Debug)]
pub struct McJsonTextField<const MAX_SIZE: usize>;
impl<const MAX_SIZE: usize> McReadBuf for McJsonTextField<MAX_SIZE> {
    type Output = McJsonTextComponent;
    type Error = MCJsonTextFieldError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let len_varint = McVarInt::read_from_buf(buf)?;
        let length = len_varint.0;
        if length < 0 {
            return Err(MCJsonTextFieldError::InvalidLength(length));
        }
        let length = length as usize;

        if length > MAX_SIZE * 3 {
            return Err(MCJsonTextFieldError::TooLong {
                actual: length,
                max: MAX_SIZE * 3,
            });
        }

        if buf.remaining() < length {
            return Err(MCJsonTextFieldError::UnexpectedEof);
        }

        let json_bytes = buf.split_to(length);
        let value: serde_json::Value = serde_json::from_slice(&json_bytes)?;
        Ok(McJsonTextComponent(value))
    }
}
