use super::{McReadBuf, McVarInt, McVarIntError};
use crate::connection::s2c::ParsePacketError;
use bytes::Bytes;
use std::marker::PhantomData;

#[derive(Debug, thiserror::Error)]
pub enum McPrefixedArrayError {
    #[error("McPrefixedArray error: failed to read array length: {0}")]
    LengthError(#[from] McVarIntError),

    #[error("McPrefixedArray error: array length cannot be negative: {0}")]
    NegativeLength(i32),

    #[error("McPrefixedArray error: failed to parse element at index {index}: {err}")]
    ElementError {
        index: usize,
        #[source]
        err: Box<ParsePacketError>,
    },
}

const MAX_PREALLOCATE_SIZE: usize = 1024;
pub struct McPrefixedArrayField<P>(PhantomData<P>);

impl<P: McReadBuf> McReadBuf for McPrefixedArrayField<P>
where
    P::Error: Into<ParsePacketError>,
{
    type Output = Vec<P::Output>;
    type Error = McPrefixedArrayError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let length = McVarInt::read_from_buf(buf)?;
        let length = match length.with_min_check(0) {
            Ok(value) => Ok(value.0),
            Err(_) => Err(McPrefixedArrayError::NegativeLength(length.0)),
        }? as usize;

        let mut vec = Vec::with_capacity(length.min(MAX_PREALLOCATE_SIZE)); // OOM protection
        for i in 0..(length) {
            let elem = P::read_from_buf(buf).map_err(|err| McPrefixedArrayError::ElementError {
                index: i,
                err: Box::new(err.into()),
            })?;
            vec.push(elem);
        }
        Ok(vec)
    }
}
