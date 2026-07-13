use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField};
use bytes::Bytes;

#[derive(Debug)]
pub struct CookieRequestPacket {
    pub key: String,
}
impl ClientBoundPacket for CookieRequestPacket {
    const MC_NAME: &str = "cookie_request";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let key = McStringField::<32767>::read_from_buf(&mut data)?;
        Ok(Self { key })
    }
}
