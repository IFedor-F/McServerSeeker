use super::{McBool, McPrefixedArrayField, McReadBuf, McStringField};
use crate::connection::s2c::ParsePacketError;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct GameProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

impl McReadBuf for GameProfileProperty {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let name = McStringField::<32767>::read_from_buf(buf)?;
        let value = McStringField::<32767>::read_from_buf(buf)?;
        let signature = if McBool::read_from_buf(buf)? {
            Some(McStringField::<32767>::read_from_buf(buf)?)
        } else {
            None
        };
        Ok(Self {
            name,
            value,
            signature,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GameProfile {
    pub name: String,
    pub properties: Vec<GameProfileProperty>,
}

impl McReadBuf for GameProfile {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let name = McStringField::<16>::read_from_buf(buf)?;
        let properties = McPrefixedArrayField::<GameProfileProperty>::read_from_buf(buf)?;
        Ok(Self { name, properties })
    }
}
