use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound)
#[derive(Debug)]
pub struct PluginMessagePacket {
    pub channel: String,
    pub data: Bytes,
}
impl ClientBoundPacket for PluginMessagePacket {
    const MC_NAME: &str = "custom_payload";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let channel = McStringField::<32767>::read_from_buf(&mut data)?;
        Ok(Self { channel, data })
    }
}
