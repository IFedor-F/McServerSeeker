use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
#[error("Expected UUID v3 or v4")]
pub struct PlayerInvalidUUID;

/// Represents Minecraft Player
///
/// If is_online is true, uuid is random (v4), else uuid is based on player's name
#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub uuid: uuid::Uuid,
    pub is_online: bool,
}
impl Player {
    pub fn from_str(name: &str, uuid_string: &str) -> Result<Player, PlayerInvalidUUID> {
        match uuid::Uuid::from_str(uuid_string) {
            Ok(uuid) => match uuid.get_version() {
                Some(uuid::Version::Md5) => Ok(Player {
                    name: name.to_owned(),
                    uuid,
                    is_online: false,
                }),
                Some(uuid::Version::Random) => Ok(Self {
                    name: name.to_owned(),
                    uuid,
                    is_online: true,
                }),
                _ => Err(PlayerInvalidUUID),
            },
            Err(_) => Err(PlayerInvalidUUID),
        }
    }
    pub fn from_offline_name(name: String) -> Self {
        let data = format!("OfflinePlayer:{}", name);
        let hash = md5::compute(data.as_bytes());
        let mut builder = uuid::Builder::from_md5_bytes(hash.0);
        builder
            .set_variant(uuid::Variant::RFC4122)
            .set_version(uuid::Version::Md5);
        let uuid = builder.into_uuid();

        Player {
            name,
            is_online: false,
            uuid,
        }
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
            is_online: true,
            uuid,
        }
    }
    pub fn random_like_offline() -> Self {
        const NAME_LENGTH: usize = 8;
        let name: String = std::iter::repeat_with(fastrand::alphanumeric)
            .take(NAME_LENGTH)
            .collect();
        Self::from_offline_name(name)
    }
}
