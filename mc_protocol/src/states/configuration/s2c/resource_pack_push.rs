use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McBool, McChat, McReadBuf, McStringField};
use bytes::{Buf, Bytes};
use uuid::Uuid;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Add_Resource_Pack
#[derive(Debug)]
pub struct ResourcePackPushPacket {
    pub uuid: Option<Uuid>,
    pub url: String,
    pub hash: String,
    pub forced: Option<bool>,
    pub prompt_message: Option<McChat>,
}
impl ClientBoundPacket for ResourcePackPushPacket {
    const MC_NAME: &str = "resource_pack_push";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let uuid = match protocol {
            ..=764 => None,
            765.. => Some(Uuid::from_u128(data.try_get_u128()?)),
        };
        let url = McStringField::<32767>::read_from_buf(&mut data)?;
        let hash = McStringField::<40>::read_from_buf(&mut data)?;
        let (forced, prompt_message) = match protocol {
            ..=754 => (None, None),
            755.. => {
                let forced = Some(McBool::read_from_buf(&mut data)?);
                let has_prompt = McBool::read_from_buf(&mut data)?;
                let prompt_message = if has_prompt {
                    Some(McChat::read_from_buf_with_protocol(&mut data, protocol)?)
                } else {
                    None
                };
                (forced, prompt_message)
            }
        };
        Ok(Self {
            uuid,
            url,
            hash,
            forced,
            prompt_message,
        })
    }
}
