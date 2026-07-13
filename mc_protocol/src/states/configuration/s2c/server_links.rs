use super::types::BuildInLabel;
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{
    McBool, McJsonTextComponent, McJsonTextField, McPrefixedArrayField, McReadBuf, McStringField,
    McVarInt,
};
use bytes::Bytes;

#[derive(Debug)]
pub enum ServerLinkLabel {
    BuiltIn(BuildInLabel),
    Custom(McJsonTextComponent),
}

#[derive(Debug)]
pub struct ServerLink {
    pub label: ServerLinkLabel,
    pub url: String,
}
impl McReadBuf for ServerLink {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(data: &mut Bytes) -> Result<Self, ParsePacketError> {
        let is_built_in = McBool::read_from_buf(data)?;
        let label = match is_built_in {
            true => {
                let enum_id = McVarInt::read_from_buf(data)?.with_check(0, 9)?.0;
                ServerLinkLabel::BuiltIn(BuildInLabel::from(enum_id))
            }
            false => {
                let text = McJsonTextField::<262144>::read_from_buf(data)?;
                ServerLinkLabel::Custom(text)
            }
        };
        let url = McStringField::<32767>::read_from_buf(data)?;
        Ok(Self { label, url })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Server_Links
#[derive(Debug)]
pub struct ServerLinksPacket {
    pub links: Vec<ServerLink>,
}

impl ClientBoundPacket for ServerLinksPacket {
    const MC_NAME: &str = "server_links";

    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let links = McPrefixedArrayField::<ServerLink>::read_from_buf(&mut data)?;
        Ok(Self { links })
    }
}
