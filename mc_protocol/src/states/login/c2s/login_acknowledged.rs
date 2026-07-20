use super::ServerBoundPacket;
use bytes::BytesMut;

#[derive(Debug)]
pub struct LoginAcknowledgedPacket;
impl ServerBoundPacket for LoginAcknowledgedPacket {
    const MC_NAME: &'static str = "login_acknowledged";
    fn encode_payload(self, _: &mut BytesMut, _: i32) {}
}
