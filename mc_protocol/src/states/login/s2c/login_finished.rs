use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{GameProfile, McReadBuf, McStringField};
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug)]
pub struct LoginFinishedPacket {
    pub uuid: Uuid,
    pub game_profile: GameProfile,
    pub session_uuid: Option<Uuid>,
}
impl ClientBoundPacket for LoginFinishedPacket {
    const MC_NAME: &str = "login_finished";
    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        match protocol {
            ..=734 => {
                let uuid = Uuid::try_parse(&McStringField::<36>::read_from_buf(&mut data)?)
                    .map_err(|_| ParsePacketError::InvalidStringUuid)?;
                Ok(Self {
                    uuid,
                    game_profile: GameProfile {
                        name: McStringField::<16>::read_from_buf(&mut data)?,
                        properties: vec![],
                    },
                    session_uuid: None,
                })
            }
            735..=758 => Ok(Self {
                uuid: Uuid::from_u128(data.try_get_u128()?),
                game_profile: GameProfile {
                    name: McStringField::<16>::read_from_buf(&mut data)?,
                    properties: vec![],
                },
                session_uuid: None,
            }),

            759..=775 => Ok(Self {
                uuid: Uuid::from_u128(data.try_get_u128()?),
                game_profile: GameProfile::read_from_buf(&mut data)?,
                session_uuid: None,
                // There is also in 766 and 767 protocol field strict_error_handling, which is unused by vanilla client and being removed after. We didn't parse this.
            }),
            776.. => Ok(Self {
                uuid: Uuid::from_u128(data.try_get_u128()?), // in Minecraft Wiki this field in GameProfile type now
                game_profile: GameProfile::read_from_buf(&mut data)?,
                session_uuid: Some(Uuid::from_u128(data.try_get_u128()?)),
            }),
        }
    }
}
