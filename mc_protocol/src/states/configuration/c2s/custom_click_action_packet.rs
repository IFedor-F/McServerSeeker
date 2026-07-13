use super::ServerBoundPacket;
use crate::types::mc_string;
use bytes::{Bytes, BytesMut};

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Click_Action
#[derive(Debug)]
pub struct CustomClickActionPacket {
    pub id: String,
    pub payload: Bytes,
}
impl ServerBoundPacket for CustomClickActionPacket {
    const MC_NAME: &'static str = "custom_click_action";
    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
        mc_string::write_to_buf(&self.id, buf);
        buf.extend(self.payload);
    }
}
