use super::ServerBoundPacket;
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct PingRequestPacket {
    timestamp: i64, // ms from launching minecraft client
}
impl ServerBoundPacket for PingRequestPacket {
    const MC_NAME: &'static str = "ping_request";
    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
        buf.put_i64(self.timestamp)
    }
}

impl PingRequestPacket {
    pub fn like_random() -> Self {
        // generates from 1 minute to 1 hour (realistic numbers)
        let timestamp = fastrand::i64(60000..3600000);
        Self { timestamp }
    }
}
