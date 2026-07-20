use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McReadBuf, McStringField};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Code_of_Conduct
#[derive(Debug)]
pub struct CodeOfConductPacket {
    pub code_of_conduct: String,
}
impl ClientBoundPacket for CodeOfConductPacket {
    const MC_NAME: &str = "code_of_conduct";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let code_of_conduct = McStringField::<32767>::read_from_buf(&mut data)?;
        Ok(Self { code_of_conduct })
    }
}
