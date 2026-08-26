use crate::types::{McReadBuf, McVarInt, McVarIntError};
use bytes::{Buf, BufMut, Bytes};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McStringFieldError {
    #[error("McString error: size ({actual} bytes) is more than limit ({max} bytes)")]
    TooLong { max: i32, actual: i32 },

    #[error("McString error: length of string ({0}) can't be less than zero")]
    InvalidLength(i32),

    #[error("McString error: not enough bytes in the buffer to read a string")]
    UnexpectedEof,

    #[error("McVarInt error while read McString: {0}")]
    InvalidVarInt(McVarIntError),

    #[error("UTF-8 parsing error while reading McString: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Null byte in string isn't allowed")]
    NullByte,
}
impl From<McVarIntError> for McStringFieldError {
    fn from(err: McVarIntError) -> Self {
        match err {
            McVarIntError::TooBig { max, actual } => McStringFieldError::TooLong { max, actual },
            McVarIntError::TooSmall { min: _, actual } => McStringFieldError::InvalidLength(actual),
            other_err => McStringFieldError::InvalidVarInt(other_err),
        }
    }
}
#[derive(Debug)]
pub struct McStringField<const MAX_SIZE: usize>;

impl<const MAX_SIZE: usize> McReadBuf for McStringField<MAX_SIZE> {
    type Output = String;
    type Error = McStringFieldError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let length = McVarInt::read_from_buf(buf)?
            .with_check(0, MAX_SIZE as i32)?
            .0 as usize;

        if buf.remaining() < length {
            return Err(McStringFieldError::UnexpectedEof);
        }

        let string_bytes = buf.split_to(length);
        let s = String::from_utf8(string_bytes.to_vec())?;
        if s.contains("\0") {
            return Err(McStringFieldError::NullByte);
        }
        Ok(s)
    }
}

pub fn write_to_buf<B: BufMut>(s: &str, buf: &mut B) {
    let len_varint = McVarInt::new(s.len() as i32);
    len_varint.write_to_buf(buf);
    buf.put_slice(s.as_bytes());
}
