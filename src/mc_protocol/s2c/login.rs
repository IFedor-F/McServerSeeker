use super::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError, try_split_to};
use crate::impl_clientbound_state;
use crate::mc_protocol::McPacket;
use crate::mc_protocol::types::bool::McBool;
use crate::mc_protocol::types::json_text_component::{JsonTextComponent, MCJsonField};
use crate::mc_protocol::types::mc_string::McStringField;
use crate::mc_protocol::types::varint::McVarInt;
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug)]
pub enum PacketEnum {
    Disconnect(DisconnectPacket),
    EncryptionRequest(EncryptionRequestPacket),
    LoginSuccess(LoginSuccessPacket),
    SetCompression(SetCompressionPacket),
    LoginPluginRequest(LoginPluginRequestPacket),
    CookieRequest(CookieRequestPacket),
}

impl_clientbound_state!(
    PacketEnum, "Login",
    Disconnect => DisconnectPacket,
    EncryptionRequest => EncryptionRequestPacket,
    LoginSuccess => LoginSuccessPacket,
    SetCompression => SetCompressionPacket,
    LoginPluginRequest => LoginPluginRequestPacket,
    CookieRequest => CookieRequestPacket,
);

#[derive(Debug)]
pub struct DisconnectPacket {
    pub reason: JsonTextComponent,
}
impl ClientBoundPacket for DisconnectPacket {
    const ID: i32 = 0x00;
    const MC_NAME: &str = "login_disconnect";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let json_text_str = MCJsonField(262144).read_from_buf(&mut data)?;
        Ok(Self {
            reason: json_text_str,
        })
    }
}

#[derive(Debug)]
pub struct EncryptionRequestPacket {
    pub server_id: Option<String>,
    pub public_key: Bytes,
    pub verify_token: Bytes,
    pub should_authenticate: bool,
}
impl ClientBoundPacket for EncryptionRequestPacket {
    const ID: i32 = 0x01;
    const MC_NAME: &str = "hello";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let server_id = match McStringField(20).read_from_buf(&mut data)?.as_str() {
            "" => None,
            s => Some(String::from(s)),
        };
        let len_public_key = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
        let public_key = try_split_to(&mut data, len_public_key)?;
        let verify_token_len = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
        let verify_token = try_split_to(&mut data, verify_token_len)?;
        let should_authenticate = McBool::read_from_buf(&mut data)?.0;

        Ok(Self {
            server_id,
            public_key,
            verify_token,
            should_authenticate,
        })
    }
}

#[derive(Debug)]
pub struct LoginSuccessPacket {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Bytes,
}
impl ClientBoundPacket for LoginSuccessPacket {
    const ID: i32 = 0x02;
    const MC_NAME: &str = "login_finished";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let uuid = Uuid::from_u128(data.try_get_u128()?);
        let username = McStringField(16).read_from_buf(&mut data)?;
        Ok(Self {
            uuid,
            username,
            properties: data,
        })
    }
}

#[derive(Debug)]
pub struct SetCompressionPacket {
    pub threshold: i32,
}
impl ClientBoundPacket for SetCompressionPacket {
    const ID: i32 = 0x03;
    const MC_NAME: &str = "login_compression";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let threshold = McVarInt::read_from_buf(&mut data)?.0;
        Ok(Self { threshold })
    }
}

#[derive(Debug)]
pub struct LoginPluginRequestPacket {
    pub message_id: i32,
    pub channel: String,
    pub data: Bytes,
}
impl ClientBoundPacket for LoginPluginRequestPacket {
    const ID: i32 = 0x04;
    const MC_NAME: &str = "custom_query";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let message_id = McVarInt::read_from_buf(&mut data)?.0;
        let channel = McStringField(32767).read_from_buf(&mut data)?;
        Ok(Self {
            message_id,
            channel,
            data,
        })
    }
}

#[derive(Debug)]
pub struct CookieRequestPacket {
    pub key: String,
}
impl ClientBoundPacket for CookieRequestPacket {
    const ID: i32 = 0x05;
    const MC_NAME: &str = "cookie_request";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let key = McStringField(32767).read_from_buf(&mut data)?;
        Ok(Self { key })
    }
}
