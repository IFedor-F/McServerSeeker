use crate::connection::s2c::ParsePacketError;
use crate::types::{McReadBuf, McVarInt};
use bytes::Bytes;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}
impl Default for GameMode {
    fn default() -> Self {
        Self::Survival
    }
}

impl TryFrom<u8> for GameMode {
    type Error = ParsePacketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Survival),
            1 => Ok(Self::Survival),
            2 => Ok(Self::Adventure),
            3 => Ok(Self::Spectator),
            n => Err(Self::Error::InvalidEnumIndex(n as usize)),
        }
    }
}
impl McReadBuf for GameMode {
    type Output = GameMode;
    type Error = ParsePacketError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let mode = McVarInt::read_from_buf(buf)?;
        Self::try_from(mode.0 as u8)
    }
}
