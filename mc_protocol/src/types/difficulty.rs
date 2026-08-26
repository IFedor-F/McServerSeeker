use crate::connection::s2c::ParsePacketError;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Difficulty {
    Peaceful = 0,
    Easy = 1,
    Normal = 2,
    Hard = 3,
}
impl TryFrom<i32> for Difficulty {
    type Error = ParsePacketError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Peaceful),
            1 => Ok(Self::Easy),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Hard),
            n => Err(Self::Error::InvalidEnumIndex(n as usize)),
        }
    }
}
impl TryFrom<u8> for Difficulty {
    type Error = ParsePacketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Peaceful),
            1 => Ok(Self::Easy),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Hard),
            n => Err(Self::Error::InvalidEnumIndex(n as usize)),
        }
    }
}
