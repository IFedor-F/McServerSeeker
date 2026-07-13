use super::ServerBoundPacket;
use crate::types::{McVarInt, mc_string};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub struct CookieResponsePacket {
    pub key: String,
    pub payload: Option<Bytes>, // Payload is bytes, without prefixed size
}
impl CookieResponsePacket {
    pub fn empty_payload(key: String) -> Self {
        Self { key, payload: None }
    }
}
impl ServerBoundPacket for CookieResponsePacket {
    const MC_NAME: &'static str = "cookie_response";

    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
        mc_string::write_to_buf(&self.key, buf);
        match self.payload {
            None => {
                buf.put_u8(0); // length of data is 0
            }
            Some(payload) => {
                McVarInt(payload.len() as i32).write_to_buf(buf);
                buf.extend(payload);
            }
        }
    }
}
