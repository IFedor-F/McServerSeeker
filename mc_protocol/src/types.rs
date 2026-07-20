pub mod varint;
pub use varint::{McVarInt, McVarIntError};
pub mod bool;
pub use bool::{McBool, McBoolError};
pub mod mc_string;
pub use mc_string::{McStringField, McStringFieldError};
pub mod prefixed_array;
pub use prefixed_array::{McPrefixedArrayError, McPrefixedArrayField};
pub mod json_text_component;
pub use json_text_component::{MCJsonTextFieldError, McJsonTextComponent, McJsonTextField};
pub mod mc_nbt;
pub use mc_nbt::{McLegacyNbtField, McModernNbtField, McNbtFieldError, McNbtTag, McTextComponent};
pub mod game_profile;
pub use game_profile::{GameProfile, GameProfileProperty};
pub mod player;
pub use player::{Player, PlayerInvalidUUID};
pub mod mc_version;
pub use mc_version::{McVersion, McVersionEnum};
pub mod chat;
pub use chat::{McChat, McChatError};

pub(crate) trait McReadBuf {
    type Output;
    type Error;
    fn read_from_buf(buf: &mut bytes::Bytes) -> Result<Self::Output, Self::Error>;
}
