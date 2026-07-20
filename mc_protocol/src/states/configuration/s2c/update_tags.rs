use super::types::{RegistryTags, Tag};
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McPrefixedArrayField, McReadBuf};
use bytes::Bytes;

/// Minecraft `update_tags` packet
///
/// Before protocol 755 `tag_type` was predefined in packet. To ensure compatibility with the new protocol, it will be packaged in the same way as in the new versions
#[derive(Debug)]
pub struct UpdateTagsPacket {
    pub registries: Vec<RegistryTags>,
}

impl ClientBoundPacket for UpdateTagsPacket {
    const MC_NAME: &str = "update_tags";

    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let registries = match protocol {
            ..=476 => {
                // Old format:
                let block_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                let item_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                let fluid_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                vec![
                    RegistryTags {
                        tag_type: String::from("minecraft:block"),
                        tags: block_tags,
                    },
                    RegistryTags {
                        tag_type: String::from("minecraft:item"),
                        tags: item_tags,
                    },
                    RegistryTags {
                        tag_type: String::from("minecraft:fluid"),
                        tags: fluid_tags,
                    },
                ]
            }
            477..=754 => {
                // Olf format + entity_tags
                let block_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                let item_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                let fluid_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                let entity_tags = McPrefixedArrayField::<Tag>::read_from_buf(&mut data)?;
                vec![
                    RegistryTags {
                        tag_type: String::from("minecraft:block"),
                        tags: block_tags,
                    },
                    RegistryTags {
                        tag_type: String::from("minecraft:item"),
                        tags: item_tags,
                    },
                    RegistryTags {
                        tag_type: String::from("minecraft:fluid"),
                        tags: fluid_tags,
                    },
                    RegistryTags {
                        tag_type: String::from("minecraft:entity_type"),
                        tags: entity_tags,
                    },
                ]
            }
            755.. => {
                // New format, tag_type isn't hardcoded
                McPrefixedArrayField::<RegistryTags>::read_from_buf(&mut data)?
            }
        };

        Ok(Self { registries })
    }
}
