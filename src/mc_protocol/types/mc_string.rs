use crate::mc_protocol::types::varint::{McVarInt, McVarIntError};
use bytes::{Buf, BufMut};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Error, Debug)]
pub enum MCStringFieldError {
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
impl From<McVarIntError> for MCStringFieldError {
    fn from(err: McVarIntError) -> Self {
        match err {
            McVarIntError::Io(io_err) => MCStringFieldError::Io(io_err),
            McVarIntError::UnexpectedEof => MCStringFieldError::UnexpectedEof,
            McVarIntError::TooBig { max, actual } => MCStringFieldError::TooLong { max, actual },
            McVarIntError::TooSmall { min: _, actual } => MCStringFieldError::InvalidLength(actual),
            other_err => MCStringFieldError::InvalidVarInt(other_err),
        }
    }
}
impl From<std::io::Error> for MCStringFieldError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(err)
        }
    }
}


#[derive(Debug)]
pub struct McStringField(pub usize);

impl McStringField {
    pub async fn read_from_stream<R: AsyncRead + Unpin>(
        &self,
        mut stream: R,
    ) -> Result<String, MCStringFieldError> {
        let len_varint = McVarInt::read_from_stream(&mut stream).await?;
        let length = len_varint.with_check(0, self.0 as i32)?.0 as usize;

        let mut buf = vec![0u8; length];
        stream.read_exact(&mut buf).await?;

        let s = String::from_utf8_lossy(&buf).into_owned();
        Ok(s)
    }
    pub fn read_from_buf(&self, buf: &mut bytes::Bytes) -> Result<String, MCStringFieldError> {
        let length = McVarInt::read_from_buf(buf)?.with_check(0, self.0 as i32)?.0 as usize;

        if buf.remaining() < length {
            return Err(MCStringFieldError::UnexpectedEof);
        }

        let string_bytes = buf.split_to(length);
        let s = String::from_utf8(string_bytes.to_vec())?;
        Ok(s)
    }

    pub fn write_to_buf<B: BufMut>(s: &str, buf: &mut B) {
        let len_varint = McVarInt::new(s.len() as i32);
        len_varint.write_to_buf(buf);
        buf.put_slice(s.as_bytes());
    }
}
