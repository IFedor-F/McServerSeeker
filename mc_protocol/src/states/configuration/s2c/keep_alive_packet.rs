use super::{ClientBoundPacket, ParsePacketError};
use bytes::{Buf, Bytes};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound)
#[derive(Debug)]
pub struct KeepAlivePacket {
    pub keep_alive_id: i64,
}
impl ClientBoundPacket for KeepAlivePacket {
    const MC_NAME: &str = "keep_alive";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let keep_alive_id = data.try_get_i64()?;
        Ok(Self { keep_alive_id })
    }
}
