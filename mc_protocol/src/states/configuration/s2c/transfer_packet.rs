use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField, McVarInt};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Transfer
#[derive(Debug)]
pub struct TransferPacket {
    pub host: String,
    pub port: i32,
}
impl ClientBoundPacket for TransferPacket {
    const MC_NAME: &str = "transfer";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let host = McStringField::<32767>::read_from_buf(&mut data)?;
        let port = McVarInt::read_from_buf(&mut data)?.0;
        Ok(Self { host, port })
    }
}
