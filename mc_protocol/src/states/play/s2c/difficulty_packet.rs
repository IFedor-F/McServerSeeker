use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McBool, McReadBuf, McVarInt};
use bytes::Bytes;
use super::types::Difficulty;

#[derive(Debug)]
pub struct ChangeDifficultyPacket {
    pub difficulty: Difficulty,
    pub locked: Option<bool>,
}
impl ClientBoundPacket for ChangeDifficultyPacket {
    const MC_NAME: &str = "change_difficulty";

    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let difficulty = McVarInt::read_from_buf(&mut data)?.with_check(0, 3)?.0;
        let difficulty = Difficulty::from(difficulty);
        let locked = match protocol {
            ..=476 => None,
            477.. => Some(McBool::read_from_buf(&mut data)?),
        };
        Ok(Self { difficulty, locked })
    }
}
