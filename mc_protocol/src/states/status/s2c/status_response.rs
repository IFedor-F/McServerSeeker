use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McJsonTextComponent, McReadBuf, McVarInt};
use bytes::Bytes;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlayersInfo {
    pub max: i32,
    pub online: i32,
    pub sample: Option<Vec<PlayerSample>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct StatusResponsePacket {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: McJsonTextComponent,
    pub favicon: Option<String>,
}
impl ClientBoundPacket for StatusResponsePacket {
    const MC_NAME: &str = "status_response";

    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let _len_json = McVarInt::read_from_buf(&mut data)?.0;
        if data.len() > 32767 {
            return Err(ParsePacketError::TooLongField {
                max_expected: 32767,
                actual: data.len(),
            });
        }
        let status: StatusResponsePacket = serde_json::from_slice(&data)?;
        Ok(status)
    }
}
