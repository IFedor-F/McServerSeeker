use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use crate::impl_serverbound_state;
use crate::types::{McVarInt, mc_string};
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub enum C2SHandshakeState {
    Handshake(HandshakePacket),
}

impl_serverbound_state! {
    state = "handshake";
    enum C2SHandshakeState;

    match protocol {
        0.. => {
            0x00 => Handshake: HandshakePacket,
        }
    }
}
#[derive(Debug)]
pub struct HandshakePacket {
    pub server_field: String,
    pub port_field: u16,
    pub protocol: i32,
    pub next_state: i32,
}
impl ServerBoundPacket for HandshakePacket {
    const MC_NAME: &'static str = "intention";

    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
        McVarInt::new(self.protocol).write_to_buf(buf);
        mc_string::write_to_buf(&self.server_field, buf);
        buf.put_u16(self.port_field);
        McVarInt::new(self.next_state).write_to_buf(buf);
    }
}
