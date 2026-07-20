use super::ServerBoundPacket;
use bytes::BytesMut;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Finish_Configuration
#[derive(Debug)]
pub struct AckFinishConfigurationPacket;
impl ServerBoundPacket for AckFinishConfigurationPacket {
    const MC_NAME: &'static str = "finish_configuration";
    fn encode_payload(self, _: &mut BytesMut, _: i32) {}
}
