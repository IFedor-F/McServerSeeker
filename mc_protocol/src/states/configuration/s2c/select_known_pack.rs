use super::types::KnownPack;
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McPrefixedArrayField, McReadBuf};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(clientbound)
#[derive(Debug)]
pub struct KnownPacksPacket {
    pub known_packs: Vec<KnownPack>,
}

impl ClientBoundPacket for KnownPacksPacket {
    const MC_NAME: &str = "select_known_packs";

    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let known_packs = McPrefixedArrayField::<KnownPack>::read_from_buf(&mut data)?;
        Ok(Self { known_packs })
    }
}
