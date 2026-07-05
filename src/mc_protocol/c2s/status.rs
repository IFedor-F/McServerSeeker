use super::ServerBoundPacket;
use crate::mc_protocol::types::McVarInt;
use bytes::BytesMut;

pub struct StatusRequestPacket {}
impl ServerBoundPacket for StatusRequestPacket {
    const ID: i32 = 0;
    const MC_NAME: &'static str = "status_request";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
    }
}
