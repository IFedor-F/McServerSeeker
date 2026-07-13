use super::ServerBoundPacket;
use bytes::BytesMut;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Accept_Code_of_Conduct
#[derive(Debug)]
pub struct AcceptCodeOfConductPacket;
impl ServerBoundPacket for AcceptCodeOfConductPacket {
    const MC_NAME: &'static str = "accept_code_of_conduct";
    fn encode_payload(self, _: &mut BytesMut, _: i32) {}
}
