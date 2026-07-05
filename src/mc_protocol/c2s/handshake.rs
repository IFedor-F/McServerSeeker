use super::ServerBoundPacket;
use crate::mc_protocol::types::mc_string::McStringField;
use bytes::{BufMut, BytesMut};

use crate::mc_protocol::types::varint::McVarInt;

pub struct HandshakePacket {
    pub server_field: String,
    pub port_field: u16,
    pub protocol: i32,
    pub next_state: i32,
}
impl ServerBoundPacket for HandshakePacket {
    const ID: i32 = 0x00;
    const MC_NAME: &'static str = "intention";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt::new(Self::ID).write_to_buf(buf);
        McVarInt::new(self.protocol).write_to_buf(buf);
        McStringField::write_to_buf(&self.server_field, buf);
        buf.put_u16(self.port_field);
        McVarInt::new(self.next_state).write_to_buf(buf);
    }
}
