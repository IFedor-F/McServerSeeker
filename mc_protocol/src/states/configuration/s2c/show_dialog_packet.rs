use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McNbtTag, mc_nbt};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Show_Dialog
#[derive(Debug)]
pub struct ShowDialogPacket {
    pub dialog: McNbtTag,
}
impl ClientBoundPacket for ShowDialogPacket {
    const MC_NAME: &str = "show_dialog";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let dialog = mc_nbt::read_nbt_from_buf(&mut data, protocol)?;
        Ok(Self { dialog })
    }
}
