use bytes::{Buf, BufMut};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Error)]
pub enum McBoolError {
    #[error("Expected 0 or 1, but got {0:b}")]
    Invalid(u8),

    #[error("Unexpected Eof")]
    UnexpectedEof,

    #[error("IO error: {0}")]
    Io(std::io::Error),
}
impl From<std::io::Error> for McBoolError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(err)
        }
    }
}
impl From<bytes::TryGetError> for McBoolError {
    fn from(_: bytes::TryGetError) -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct McBool(pub bool);

impl McBool {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub async fn read_from_stream<R: AsyncRead + Unpin>(
        mut stream: R,
    ) -> Result<Self, McBoolError> {
        let value = stream.read_u8().await?;
        match value {
            0 => Ok(Self(false)),
            1 => Ok(Self(true)),
            value => Err(McBoolError::Invalid(value)),
        }
    }

    #[inline(always)]
    pub fn read_from_buf<B: Buf>(buf: &mut B) -> Result<Self, McBoolError> {
        let value = buf.get_u8();
        match value {
            0 => Ok(Self(false)),
            1 => Ok(Self(true)),
            value => Err(McBoolError::Invalid(value)),
        }
    }

    #[inline(always)]
    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(self.0 as u8)
    }
}
