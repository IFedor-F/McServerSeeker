use super::{ClientBoundPacket, ParsePacketError};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Reset_Chat
#[derive(Debug)]
pub struct ResetChatPacket {}
impl ClientBoundPacket for ResetChatPacket {
    const MC_NAME: &str = "reset_chat";
    fn parse(_data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        Ok(Self {})
    }
}
