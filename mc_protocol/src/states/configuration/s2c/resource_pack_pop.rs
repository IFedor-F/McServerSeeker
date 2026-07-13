use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McBool, McReadBuf};
use bytes::{Buf, Bytes};
use uuid::Uuid;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Resource_Pack
#[derive(Debug)]
pub struct RemoveResourcePackPacket {
    pub uuid: Option<Uuid>,
}
impl ClientBoundPacket for RemoveResourcePackPacket {
    const MC_NAME: &str = "resource_pack_pop";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let has_uuid = McBool::read_from_buf(&mut data)?;
        let uuid = if has_uuid {
            Some(Uuid::from_u128(data.try_get_u128()?))
        } else {
            None
        };
        Ok(Self { uuid })
    }
}
