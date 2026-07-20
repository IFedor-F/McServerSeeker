use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField, McVarInt};
use bytes::Bytes;

#[derive(Debug)]
pub struct CustomQueryPacket {
    pub message_id: i32,
    pub channel: String,
    pub data: Bytes,
}
impl ClientBoundPacket for CustomQueryPacket {
    const MC_NAME: &str = "custom_query";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let message_id = McVarInt::read_from_buf(&mut data)?.0;
        let channel = McStringField::<32767>::read_from_buf(&mut data)?;
        Ok(Self {
            message_id,
            channel,
            data,
        })
    }
}
