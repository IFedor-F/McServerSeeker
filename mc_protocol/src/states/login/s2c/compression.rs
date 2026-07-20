use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McVarInt};
use bytes::Bytes;

#[derive(Debug)]
pub struct CompressionPacket {
    pub threshold: i32,
}
impl ClientBoundPacket for CompressionPacket {
    const MC_NAME: &str = "login_compression";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let threshold = McVarInt::read_from_buf(&mut data)?.0;
        Ok(Self { threshold })
    }
}
