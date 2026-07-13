use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McJsonTextComponent, McJsonTextField, McReadBuf};
use bytes::Bytes;
#[derive(Debug)]
pub struct DisconnectPacket {
    pub reason: McJsonTextComponent,
}
impl ClientBoundPacket for DisconnectPacket {
    const MC_NAME: &str = "login_disconnect";
    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let json_text_str = McJsonTextField::<262144>::read_from_buf(&mut data)?;
        Ok(Self {
            reason: json_text_str,
        })
    }
}
