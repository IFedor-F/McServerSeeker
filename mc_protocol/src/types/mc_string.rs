use crate::types::{McReadBuf, McVarInt, McVarIntError};
use bytes::{Buf, BufMut, Bytes};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Error, Debug)]
pub enum McStringFieldError {
    #[error("Size ({actual} bytes) is more than limit ({max} bytes)")]
    TooLong { max: i32, actual: i32 },

    #[error("Length of string ({0}) can't be less than zero")]
    InvalidLength(i32),

    #[error("Not enough bytes in the buffer to read a string")]
    UnexpectedEof,

    #[error("VarIntError: {0}")]
    InvalidVarInt(McVarIntError),

    #[error("I/O error: {0}")]
    Io(std::io::Error),

    #[error("UTF-8 parsing error: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}
impl From<McVarIntError> for McStringFieldError {
    fn from(err: McVarIntError) -> Self {
        match err {
            McVarIntError::Io(io_err) => McStringFieldError::Io(io_err),
            McVarIntError::UnexpectedEof => McStringFieldError::UnexpectedEof,
            McVarIntError::TooBig { max, actual } => McStringFieldError::TooLong { max, actual },
            McVarIntError::TooSmall { min: _, actual } => McStringFieldError::InvalidLength(actual),
            other_err => McStringFieldError::InvalidVarInt(other_err),
        }
    }
}
impl From<std::io::Error> for McStringFieldError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(err)
        }
    }
}

#[derive(Debug)]
pub struct McStringField<const MAX_SIZE: usize>;

impl<const MAX_SIZE: usize> McStringField<MAX_SIZE> {
    pub async fn read_from_stream<R: AsyncRead + Unpin>(
        &self,
        mut stream: R,
    ) -> Result<String, McStringFieldError> {
        let len_varint = McVarInt::read_from_stream(&mut stream).await?;
        let length = len_varint.with_check(0, MAX_SIZE as i32)?.0 as usize;

        let mut buf = vec![0u8; length];
        stream.read_exact(&mut buf).await?;

        let s = String::from_utf8(buf)?;
        Ok(s)
    }
}

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
        Ok(s)
    }
}

pub fn write_to_buf<B: BufMut>(s: &str, buf: &mut B) {
    let len_varint = McVarInt::new(s.len() as i32);
    len_varint.write_to_buf(buf);
    buf.put_slice(s.as_bytes());
}
