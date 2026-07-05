use super::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;
use crate::mc_protocol::McPacket;
use crate::mc_protocol::types::json_text_component::JsonTextComponent;
use crate::mc_protocol::types::varint::McVarInt;
use bytes::{Buf, Bytes};
use serde::Deserialize;

#[derive(Debug)]
pub enum PacketEnum {
    StatusResponse(StatusResponsePacket),
    PongResponse(PongResponsePacket),
}

impl_clientbound_state!(
    PacketEnum, "Status",
    StatusResponse => StatusResponsePacket,
    PongResponse => PongResponsePacket
);

#[derive(Deserialize, Debug, Clone)]
pub struct StatusResponsePacket {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: JsonTextComponent,
    pub favicon: Option<String>,
}
impl ClientBoundPacket for StatusResponsePacket {
    const ID: i32 = 0x00;
    const MC_NAME: &str = "status_response";

    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
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

#[derive(Debug)]
pub struct PongResponsePacket {
    pub timestamp: i64, // mc long
}
impl ClientBoundPacket for PongResponsePacket {
    const ID: i32 = 0x01;
    const MC_NAME: &str = "pong_response";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let timestamp = data.try_get_i64()?;
        Ok(Self { timestamp })
    }
}
