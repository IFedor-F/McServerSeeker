use super::{ClientBoundPacket, ParsePacketError};
use bytes::Bytes;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clear_Dialog
#[derive(Debug)]
pub struct ClearDialogPacket;
impl ClientBoundPacket for ClearDialogPacket {
    const MC_NAME: &str = "clear_dialog";
    fn parse(_data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        Ok(Self)
    }
}
