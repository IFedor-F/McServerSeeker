use super::ServerBoundPacket;
use bytes::{Bytes, BytesMut};

#[derive(Debug)]
pub struct KeyPacket {
    pub shared_secret: Bytes,
    pub verify_token: Bytes,
}
impl ServerBoundPacket for KeyPacket {
    const MC_NAME: &'static str = "key";
    fn encode_payload(self, _: &mut BytesMut, _: i32) {
        unimplemented!()
    }
}
