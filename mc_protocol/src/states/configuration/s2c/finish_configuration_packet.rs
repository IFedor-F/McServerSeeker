use super::{ClientBoundPacket, ParsePacketError};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Finish_Configuration
#[derive(Debug)]
pub struct FinishConfigurationPacket {}
impl ClientBoundPacket for FinishConfigurationPacket {
    const MC_NAME: &str = "finish_configuration";
    fn parse(_data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        Ok(Self {})
    }
}
