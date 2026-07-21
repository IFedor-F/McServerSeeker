use super::types::CustomPayloadData;
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField};
use bytes::Bytes;
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound)

#[derive(Debug)]
pub struct CustomPayloadPacket {
    pub channel: String,
    pub data: CustomPayloadData,
}
impl ClientBoundPacket for CustomPayloadPacket {
    const MC_NAME: &str = "custom_payload";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let channel = McStringField::<32767>::read_from_buf(&mut data)?;
        let data = CustomPayloadData::new(&channel, data)?;
        Ok(Self { channel, data })
    }
}
