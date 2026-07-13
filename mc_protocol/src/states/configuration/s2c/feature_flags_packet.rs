use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField, McVarInt};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Feature_Flags
#[derive(Debug)]
pub struct FeatureFlagsPacket {
    pub features: Vec<String>,
}
impl ClientBoundPacket for FeatureFlagsPacket {
    const MC_NAME: &str = "update_enabled_features";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        const MAX_FEATURES: i32 = 1024; // Minecraft client doesn't have limit for features
        let count = McVarInt::read_from_buf(&mut data)?
            .with_check(0, MAX_FEATURES)?
            .0 as usize;
        let mut features = Vec::with_capacity(count);
        for _ in 0..count {
            features.push(McStringField::<32767>::read_from_buf(&mut data)?);
        }
        Ok(Self { features })
    }
}
