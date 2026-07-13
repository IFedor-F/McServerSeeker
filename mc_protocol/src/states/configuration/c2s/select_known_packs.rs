use super::types::KnownPack;
use super::ServerBoundPacket;
use crate::types::{mc_string, McVarInt};
use bytes::BytesMut;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(serverbound)
#[derive(Debug)]
pub struct SelectKnownPacksPacket {
    pub known_packs: Vec<KnownPack>,
}
impl ServerBoundPacket for SelectKnownPacksPacket {
    const MC_NAME: &'static str = "select_known_packs";
    fn encode_payload(self, buf: &mut BytesMut, _: i32) {
        McVarInt(self.known_packs.len() as i32).write_to_buf(buf);
        for pack in self.known_packs {
            mc_string::write_to_buf(&pack.namespace, buf);
            mc_string::write_to_buf(&pack.id, buf);
            mc_string::write_to_buf(&pack.version, buf);
        }
    }
}
