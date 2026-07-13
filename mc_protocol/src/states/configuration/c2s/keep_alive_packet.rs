use super::ServerBoundPacket;
use crate::types::McVarInt;
use bytes::{BufMut, BytesMut};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(serverbound)
#[derive(Debug)]
pub struct KeepAlivePacket {
    pub keep_alive_id: i64,
}
impl ServerBoundPacket for KeepAlivePacket {
    const MC_NAME: &'static str = "keep_alive";
    fn encode_payload(self, buf: &mut BytesMut, protocol: i32) {
        match protocol {
            ..=46 => {
                buf.put_i32(self.keep_alive_id as i32) // int (i32)
            }
            47..=339 => {
                McVarInt(self.keep_alive_id as i32).write_to_buf(buf) // VarInt
            }
            340.. => {
                buf.put_i64(self.keep_alive_id) // long (i64)
            }
        }
    }
}
