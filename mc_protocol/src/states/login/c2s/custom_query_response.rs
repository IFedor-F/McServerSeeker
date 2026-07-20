use super::ServerBoundPacket;
use crate::types::McVarInt;
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub struct CustomQueryAnswerPacket {
    pub message_id: i32,
    pub data: Option<Bytes>,
}
impl CustomQueryAnswerPacket {
    pub fn empty_data(message_id: i32) -> Self {
        Self {
            message_id,
            data: None,
        }
    }
}
impl ServerBoundPacket for CustomQueryAnswerPacket {
    const MC_NAME: &'static str = "custom_query_answer";
    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
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
