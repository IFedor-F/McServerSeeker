use super::McReadBuf;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Read;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Debug, Error)]
pub enum McVarIntError {
    #[error("McVarInt can't be more than 5 bytes")]
    TooLong,

    #[error("Unexpected EOF while reading McVarInt")]
    UnexpectedEof,

    #[error("IO error while reading McVarInt: {0}")]
    Io(std::io::Error),

    #[error("VarInt check failed, value is too small, min: {min}, actual: {actual}")]
    TooSmall { min: i32, actual: i32 },

    #[error("VarInt check failed, value is too big, max: {max}, actual: {actual}")]
    TooBig { max: i32, actual: i32 },
}

impl From<std::io::Error> for McVarIntError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(err)
        }
    }
}
impl From<bytes::TryGetError> for McVarIntError {
    fn from(_: bytes::TryGetError) -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct McVarInt(pub i32);

impl McVarInt {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
    #[inline(always)]
    pub fn read_from_reader<R: Read>(mut reader: R) -> Result<Self, McVarIntError> {
        let mut num_read = 0;
        let mut result = 0u32;
        let mut read = [0u8; 1];

        loop {
            if num_read == 5 {
                return Err(McVarIntError::TooLong);
            }
            reader.read_exact(&mut read)?;

            let value = (read[0] & 0b0111_1111) as u32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(McVarIntError::TooLong);
            }

            if (read[0] & 0b1000_0000) == 0 {
                break;
            }
        }
        Ok(Self(result as i32))
    }
    #[inline(always)]
    pub async fn read_from_stream<R: AsyncRead + Unpin>(
        mut stream: R,
    ) -> Result<Self, McVarIntError> {
        let mut num_read = 0;
        let mut result = 0u32;
        let mut read = [0u8; 1];

        loop {
            if num_read == 5 {
                return Err(McVarIntError::TooLong);
            }
            stream.read_exact(&mut read).await?;
            let value = (read[0] & 0b0111_1111) as u32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(McVarIntError::TooLong);
            }

            if (read[0] & 0b1000_0000) == 0 {
                break;
            }
        }
        Ok(Self(result as i32))
    }

    #[inline(always)]
    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        let mut u_val = self.0 as u32;
        loop {
            let mut temp = (u_val & 0b0111_1111) as u8;
            u_val >>= 7;
            if u_val != 0 {
                temp |= 0b1000_0000;
            }
            buf.put_u8(temp);
            if u_val == 0 {
                break;
            }
        }
    }

    #[inline(always)]
    pub fn to_buf(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.len());
        self.write_to_buf(&mut buf);
        buf.freeze()
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        let value = self.0 as u32;
        if value == 0 {
            return 1;
        }
        ((32 - value.leading_zeros() + 6) / 7) as usize
    }

    #[inline(always)]
    pub fn with_check(self, min: i32, max: i32) -> Result<Self, McVarIntError> {
        if self.0 < min {
            return Err(McVarIntError::TooSmall {
                min,
                actual: self.0,
            });
        }
        if self.0 > max {
            return Err(McVarIntError::TooBig {
                max,
                actual: self.0,
            });
        }
        Ok(self)
    }
    #[inline(always)]
    pub fn with_min_check(self, min: i32) -> Result<Self, McVarIntError> {
        if self.0 < min {
            Err(McVarIntError::TooSmall {
                min,
                actual: self.0,
            })
        } else {
            Ok(self)
        }
    }
    #[inline(always)]
    pub fn with_max_check(self, max: i32) -> Result<Self, McVarIntError> {
        if self.0 > max {
            Err(McVarIntError::TooBig {
                max,
                actual: self.0,
            })
        } else {
            Ok(self)
        }
    }
}
impl McReadBuf for McVarInt {
    type Output = Self;
    type Error = McVarIntError;

    #[inline(always)]
    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let mut num_read = 0;
        let mut result = 0u32;

        loop {
            // Prevent reading more than 5 bytes to avoid shift overflow (7 * 5 = 35 >= 32)
            if num_read == 5 {
                return Err(McVarIntError::TooLong);
            }

            // Ensure we have data before attempting to read
            if !buf.has_remaining() {
                return Err(McVarIntError::UnexpectedEof);
            }

            let read = buf.try_get_u8()?;
            let value = (read & 0b0111_1111) as u32;

            result |= value << (7 * num_read);
            num_read += 1;

            if (read & 0b1000_0000) == 0 {
                break;
            }
        }

        Ok(Self(result as i32))
    }
}
