use super::ServerBoundPacket;
use crate::mc_protocol::types::mc_string::McStringField;
use crate::mc_protocol::types::player::Player;
use crate::mc_protocol::types::varint::McVarInt;
use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

pub struct LoginStartPacket {
    pub name: String,
    pub uuid: Uuid,
}

impl LoginStartPacket {
    pub fn from_player(player: Player) -> Self {
        Self {
            name: player.name,
            uuid: player.uuid,
        }
    }
}
impl ServerBoundPacket for LoginStartPacket {
    const ID: i32 = 0x00;
    const MC_NAME: &'static str = "hello";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McStringField::write_to_buf(&self.name, buf);
        buf.extend_from_slice(self.uuid.as_bytes());
    }
}

pub struct EncryptionResponsePacket {
    pub shared_secret: Bytes,
    pub verify_token: Bytes,
}
impl ServerBoundPacket for EncryptionResponsePacket {
    const ID: i32 = 0x01;
    const MC_NAME: &'static str = "key";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        buf.extend(self.shared_secret);
        buf.extend(self.verify_token);
    }
}

pub struct LoginPluginResponsePacket<T>
where
    T: IntoIterator<Item = u8>,
{
    pub message_id: i32,
    pub data: Option<T>,
}
impl LoginPluginResponsePacket<std::iter::Empty<u8>> {
    pub fn with_empty_data(message_id: i32) -> Self {
        Self {
            message_id,
            data: None,
        }
    }
}
impl<T> ServerBoundPacket for LoginPluginResponsePacket<T>
where
    T: IntoIterator<Item = u8>,
{
    const ID: i32 = 0x02;
    const MC_NAME: &'static str = "custom_query_answer";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McVarInt(self.message_id).write_to_buf(buf);
        match self.data {
            None => {
                buf.put_u8(0);
            }
            Some(data) => {
                buf.put_u8(1);
                buf.extend(data);
            }
        }
    }
}
pub struct LoginAcknowledgedPacket;
impl ServerBoundPacket for LoginAcknowledgedPacket {
    const ID: i32 = 0x03;
    const MC_NAME: &'static str = "login_acknowledged";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
    }
}
pub struct CookieResponsePacket<T>
where
    T: IntoIterator<Item = u8>,
{
    pub key: String,
    pub payload: Option<T>,
}
impl CookieResponsePacket<std::iter::Empty<u8>> {
    pub fn empty_payload(key: String) -> Self {
        Self { key, payload: None }
    }
}
impl<T> ServerBoundPacket for CookieResponsePacket<T>
where
    T: IntoIterator<Item = u8>,
{
    const ID: i32 = 0x04;
    const MC_NAME: &'static str = "cookie_response";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McStringField::write_to_buf(&self.key, buf);
        match self.payload {
            None => {
                buf.put_u8(0);
            }
            Some(payload) => {
                buf.put_u8(1);
                buf.extend(payload);
            }
        }
    }
}
