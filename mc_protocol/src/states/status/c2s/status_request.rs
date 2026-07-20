use super::{ServerBoundPacket};
use bytes::BytesMut;

#[derive(Debug)]
pub struct StatusRequestPacket;
impl ServerBoundPacket for StatusRequestPacket {
    const MC_NAME: &'static str = "status_request";
    fn encode_payload(self, _: &mut BytesMut, _: i32) {}
}
