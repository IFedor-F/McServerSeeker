use bytes::BytesMut;

pub mod handshake;
pub mod login;
pub mod status;
pub mod configuration;

pub trait ServerBoundPacket {
    const ID: i32;
    const MC_NAME: &'static str;
    fn encode(self, buf: &mut BytesMut);
}
