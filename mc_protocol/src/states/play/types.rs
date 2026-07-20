use crate::connection::s2c::{ParsePacketError, try_split_to};
pub use crate::states::configuration::types::ResourcePackIdent;
use crate::types::{GameProfile, McReadBuf, McTextComponent, McVarInt};
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct DeathLocation {
    pub dimension_name: String,
    pub location: i64,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Difficulty {
    Peaceful = 0,
    Easy = 1,
    Normal = 2,
    Hard = 3,
}
impl From<i32> for Difficulty {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            3 => Self::Hard,
            n => panic!("Difficulty expected 0, 1, 2 or 3, but get {}", n),
        }
    }
}
impl From<u8> for Difficulty {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            3 => Self::Hard,
            n => panic!("Difficulty expected 0, 1, 2 or 3, but get {}", n),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct PlayerActions {
    pub add_player: bool,
    pub initialize_chat: bool,
    pub update_game_mode: bool,
    pub update_listed: bool,
    pub update_latency: bool,
    pub update_display_name: bool,
    pub update_list_priority: bool,
    pub update_hat: bool,
    pub remove_player: bool,
}

#[derive(Debug, Clone)]
pub struct ChatSession {
    pub session_id: Uuid,
    pub expire_time: i64,
    pub public_key: Bytes,
    pub key_signature: Bytes,
}

impl McReadBuf for ChatSession {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let session_id = Uuid::from_u128(buf.try_get_u128()?);
        let expire_time = buf.try_get_i64()?;

        let pk_len = McVarInt::read_from_buf(buf)?.with_min_check(0)?.0 as usize;
        let public_key = try_split_to(buf, pk_len)?;

        let sig_len = McVarInt::read_from_buf(buf)?.with_min_check(0)?.0 as usize;
        let key_signature = try_split_to(buf, sig_len)?;

        Ok(Self {
            session_id,
            expire_time,
            public_key,
            key_signature,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PlayerCrypto {
    pub timestamp: i64,
    pub public_key: Bytes,
    pub signature: Bytes,
}

impl McReadBuf for PlayerCrypto {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let timestamp = buf.try_get_i64()?;

        let pk_len = McVarInt::read_from_buf(buf)?.with_min_check(0)?.0 as usize;
        let public_key = try_split_to(buf, pk_len)?;

        let sig_len = McVarInt::read_from_buf(buf)?.with_min_check(0)?.0 as usize;
        let signature = try_split_to(buf, sig_len)?;

        Ok(Self {
            timestamp,
            public_key,
            signature,
        })
    }
}

#[derive(Debug)]
pub enum McChatComponent {
    String(String),
    Nbt(McTextComponent),
}

#[derive(Debug, Default)]
pub struct PlayerInfo {
    pub uuid: Option<Uuid>,

    // Add Player / Initialize Chat
    pub profile: Option<GameProfile>,
    pub chat_session: Option<ChatSession>,

    // Updates
    pub game_mode: Option<i32>,
    pub listed: Option<bool>,
    pub ping: Option<i32>,
    pub display_name: Option<McChatComponent>,
    pub list_priority: Option<i32>,
    pub show_hat: Option<bool>,

    // Crypto
    pub crypto: Option<PlayerCrypto>,
}
