use super::{ClientBoundPacket, ParsePacketError};
use bytes::{Buf, Bytes};
use crate::types::{McReadBuf, McVarInt};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound)
#[derive(Debug)]
pub struct KeepAlivePacket {
    pub keep_alive_id: i64,
}
impl ClientBoundPacket for KeepAlivePacket {
    const MC_NAME: &str = "keep_alive";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let keep_alive_id = match protocol {
            ..=46 => {
                data.try_get_i32()? as i64
            },
            47..=339 => {
                McVarInt::read_from_buf(&mut data)?.0 as i64
            },
            340.. => {
                data.try_get_i64()?
            },
        };
        Ok(Self { keep_alive_id })
    }
}
