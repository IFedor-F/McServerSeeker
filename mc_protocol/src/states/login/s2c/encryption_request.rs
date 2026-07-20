use super::{ClientBoundPacket, ParsePacketError};
use crate::connection::s2c::try_split_to;
use crate::types::{McBool, McReadBuf, McStringField, McVarInt};
use bytes::{Buf, Bytes};

#[derive(Debug)]
pub struct EncryptionRequestPacket {
    pub server_id: Option<String>,
    pub public_key: Bytes,
    pub verify_token: Bytes,
    pub should_authenticate: Option<bool>,
}
impl ClientBoundPacket for EncryptionRequestPacket {
    const MC_NAME: &str = "hello";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        match protocol {
            ..=46 => {
                let server_id = match McStringField::<20>::read_from_buf(&mut data)?.as_str() {
                    "" => None,
                    s => Some(String::from(s)),
                };
                let len_public_key = data.try_get_i16()?;
                if len_public_key < 0 {
                    return Err(ParsePacketError::NegativeLength);
                }
                let public_key = try_split_to(&mut data, len_public_key as usize)?;
                let verify_token_len = data.try_get_i16()?;
                if verify_token_len < 0 {
                    return Err(ParsePacketError::NegativeLength);
                }
                let verify_token = try_split_to(&mut data, verify_token_len as usize)?;
                Ok(Self {
                    server_id,
                    public_key,
                    verify_token,
                    should_authenticate: None,
                })
            }
            47..=765 => {
                let server_id = match McStringField::<20>::read_from_buf(&mut data)?.as_str() {
                    "" => None,
                    s => Some(String::from(s)),
                };
                let len_public_key =
                    McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let public_key = try_split_to(&mut data, len_public_key)?;
                let verify_token_len =
                    McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let verify_token = try_split_to(&mut data, verify_token_len)?;
                Ok(Self {
                    server_id,
                    public_key,
                    verify_token,
                    should_authenticate: None,
                })
            }
            766.. => {
                let server_id = match McStringField::<20>::read_from_buf(&mut data)?.as_str() {
                    "" => None,
                    s => Some(String::from(s)),
                };
                let len_public_key =
                    McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let public_key = try_split_to(&mut data, len_public_key)?;
                let verify_token_len =
                    McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let verify_token = try_split_to(&mut data, verify_token_len)?;
                let should_authenticate = Some(McBool::read_from_buf(&mut data)?);
                Ok(Self {
                    server_id,
                    public_key,
                    verify_token,
                    should_authenticate,
                })
            }
        }
    }
}
