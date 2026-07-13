use super::{ClientBoundPacket, ParsePacketError};
use crate::connection::s2c::try_split_to;
use crate::types::{McReadBuf, McStringField, McVarInt};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Store_Cookie
#[derive(Debug)]
pub struct StoreCookiePacket {
    pub key: String,
    pub payload: Bytes,
}
impl ClientBoundPacket for StoreCookiePacket {
    const MC_NAME: &str = "store_cookie";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let key = McStringField::<32767>::read_from_buf(&mut data)?;
        let payload_len = McVarInt::read_from_buf(&mut data)?.with_check(0, 5120)?.0 as usize;
        let payload = try_split_to(&mut data, payload_len)?;
        Ok(Self { key, payload })
    }
}
