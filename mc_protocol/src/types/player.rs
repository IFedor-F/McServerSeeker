use crate::states::status::s2c::status_response::PlayerSample;
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum OnlineType {
    Offline,
    Online,
    Anonymous,
}

#[derive(Debug, thiserror::Error)]
pub enum PlayerParseError {
    #[error("expected v3 or v4 uuid")]
    InvalidUUID,

    #[error("player name is too long ({0})")]
    TooLongName(usize),
}

/// Represents Minecraft Player
///
/// - If online_type is `Online`, uuid is random (v4)
/// - if online_type is `Offline` uuid is based on player's name (v3)
/// - if online_type is `Anonymous`, uuid is nil (all zeroes)
#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(try_from = "PlayerSample")]
pub struct Player {
    pub name: String,
    pub uuid: Uuid,
    pub online_type: OnlineType,
}

impl TryFrom<PlayerSample> for Player {
    type Error = PlayerParseError;
    fn try_from(sample: PlayerSample) -> Result<Self, Self::Error> {
        Player::from_strings(sample.name, sample.id)
    }
}
impl Player {
    pub fn from_strings(name: String, uuid_string: String) -> Result<Player, PlayerParseError> {
        let uuid = Uuid::from_str(&uuid_string).map_err(|_| PlayerParseError::InvalidUUID)?;
        Self::from_name_and_uuid(name, uuid)
    }
    pub fn from_name_and_uuid(name: String, uuid: Uuid) -> Result<Player, PlayerParseError> {
        if name.len() > 16 {
            return Err(PlayerParseError::TooLongName(name.len()));
        }
        match uuid.get_version() {
            Some(uuid::Version::Md5) => Ok(Player {
                name,
                uuid,
                online_type: OnlineType::Offline,
            }),
            Some(uuid::Version::Random) => Ok(Self {
                name,
                uuid,
                online_type: OnlineType::Online,
            }),
            _ => {
                if uuid.is_nil() {
                    Ok(Self {
                        name,
                        uuid,
                        online_type: OnlineType::Anonymous,
                    })
                } else {
                    Err(PlayerParseError::InvalidUUID)
                }
            }
        }
    }
    pub fn from_offline_name(name: String) -> Result<Self, PlayerParseError> {
        if name.len() > 16 {
            return Err(PlayerParseError::TooLongName(name.len()));
        }

        let data = format!("OfflinePlayer:{}", name);
        let hash = md5::compute(data.as_bytes());
        let mut builder = uuid::Builder::from_md5_bytes(hash.0);
        builder
            .set_variant(uuid::Variant::RFC4122)
            .set_version(uuid::Version::Md5);
        let uuid = builder.into_uuid();

        Ok(Player {
            name,
            online_type: OnlineType::Offline,
            uuid,
        })
    }

    /// Generate random name and random uuid (v4) for Player using fastrand (not cryptographically secure).
    ///
    /// Like online means what uuid is v4 and not based on name.
    pub fn random_like_online() -> Self {
        const NAME_LENGTH: usize = 8;
        let name: String = std::iter::repeat_with(fastrand::alphanumeric)
            .take(NAME_LENGTH)
            .collect();
        let random_uuid_bytes: [u8; 16] = std::array::from_fn(|_| fastrand::u8(..));
        let uuid = uuid::Builder::from_random_bytes(random_uuid_bytes).into_uuid();
        Player {
            name,
            online_type: OnlineType::Online,
            uuid,
        }
    }
    pub fn random_like_offline() -> Self {
        const NAME_LENGTH: usize = 8;
        let name: String = std::iter::repeat_with(fastrand::alphanumeric)
            .take(NAME_LENGTH)
            .collect();
        Self::from_offline_name(name).unwrap() // because we generate correct name (len <= 16)
    }
}
