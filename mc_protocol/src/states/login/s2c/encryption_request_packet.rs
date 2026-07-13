use super::{ClientBoundPacket, ParsePacketError};
use crate::connection::s2c::try_split_to;
use crate::types::{McBool, McReadBuf, McStringField, McVarInt};
use bytes::Bytes;

#[derive(Debug)]
pub struct EncryptionRequestPacket {
    pub server_id: Option<String>,
    pub public_key: Bytes,
    pub verify_token: Bytes,
    pub should_authenticate: bool,
}
impl ClientBoundPacket for EncryptionRequestPacket {
    const MC_NAME: &str = "hello";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let server_id = match McStringField::<20>::read_from_buf(&mut data)?.as_str() {
            "" => None,
            s => Some(String::from(s)),
        };
        let len_public_key = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
        let public_key = try_split_to(&mut data, len_public_key)?;
        let verify_token_len = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
        let verify_token = try_split_to(&mut data, verify_token_len)?;
        let should_authenticate = McBool::read_from_buf(&mut data)?;

        Ok(Self {
            server_id,
            public_key,
            verify_token,
            should_authenticate,
        })
    }
}
