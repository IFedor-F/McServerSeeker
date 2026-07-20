use super::ServerBoundPacket;
use bytes::{BufMut, BytesMut};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong
#[derive(Debug)]
pub struct PongPacket {
    pub id: i32,
}
impl ServerBoundPacket for PongPacket {
    const MC_NAME: &'static str = "pong";
    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
        buf.put_i32(self.id);
    }
}
