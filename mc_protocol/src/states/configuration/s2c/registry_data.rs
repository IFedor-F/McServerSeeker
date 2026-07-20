use super::types::{RegistryData, RegistryEntry};
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McModernNbtField, McNbtTag, McPrefixedArrayField, McReadBuf, McStringField};
use bytes::Bytes;

/// Before 766 it was single nbt, since 766 it is:
/// ```text
/// id (identifier)
/// entries (prefixed array of (entry_id (identifier) + data (nbt)))
/// ```
/// See more at [Minecraft Wiki](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Registry_Data)
#[derive(Debug)]
pub enum RegistryDataPacket {
    NbtCodec(McNbtTag),
    RegistryData(RegistryData),
}
impl ClientBoundPacket for RegistryDataPacket {
    const MC_NAME: &str = "registry_data";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        match protocol {
            ..=765 => {
                let nbt_codec = McModernNbtField::read_from_buf(&mut data)?;
                Ok(Self::NbtCodec(nbt_codec))
            }
            766.. => {
                let id = McStringField::<32767>::read_from_buf(&mut data)?;
                let entries = McPrefixedArrayField::<RegistryEntry>::read_from_buf(&mut data)?;
                let registry_data = RegistryData { id, entries };
                Ok(Self::RegistryData(registry_data))
            }
        }
    }
}
