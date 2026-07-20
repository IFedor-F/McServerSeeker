use super::{ClientBoundPacket, ParsePacketError};
use bytes::{Buf, Bytes};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping
#[derive(Debug)]
pub struct PingPacket {
    pub id: i32,
}
impl ClientBoundPacket for PingPacket {
    const MC_NAME: &str = "ping";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let id = data.try_get_i32()?;
        Ok(Self { id })
    }
}
