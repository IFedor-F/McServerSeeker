use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McJsonTextComponent, McJsonTextField, McReadBuf, McTextComponent};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect
#[derive(Debug)]
pub enum DisconnectPacket {
    TextComponentReason(McTextComponent),
    JsonTextComponent(McJsonTextComponent),
}
impl ClientBoundPacket for DisconnectPacket {
    const MC_NAME: &str = "disconnect";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        match protocol {
            ..=764 => {
                let reason = McJsonTextField::<262144>::read_from_buf(&mut data)?;
                Ok(Self::JsonTextComponent(reason))
            }
            765.. => {
                let reason = McTextComponent::read_from_buf_with_protocol(&mut data, protocol)?;
                Ok(Self::TextComponentReason(reason))
            }
        }
    }
}
