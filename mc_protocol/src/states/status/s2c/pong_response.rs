use super::{ClientBoundPacket, ParsePacketError};
use bytes::{Buf, Bytes};

#[derive(Debug)]
pub struct PongResponsePacket {
    pub timestamp: i64, // mc long
}
impl ClientBoundPacket for PongResponsePacket {
    const MC_NAME: &str = "pong_response";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let timestamp = data.try_get_i64()?;
        Ok(Self { timestamp })
    }
}
