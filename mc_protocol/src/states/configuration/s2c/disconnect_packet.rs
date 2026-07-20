use super::{ClientBoundPacket, ParsePacketError};
use crate::types::McChat;
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect
#[derive(Debug)]
pub struct DisconnectPacket {
    pub reason: McChat,
}
impl ClientBoundPacket for DisconnectPacket {
    const MC_NAME: &str = "disconnect";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let reason = McChat::read_from_buf_with_protocol(&mut data, protocol)?;
        Ok(Self { reason })
    }
}
