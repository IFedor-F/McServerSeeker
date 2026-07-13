mod commands_packet;
pub use commands_packet::CommandsPacket;
mod difficulty_packet;
pub use difficulty_packet::ChangeDifficultyPacket;
mod login_packet;
pub use login_packet::LoginPacket;
mod player_info_update;
pub use player_info_update::PlayerInfoUpdatePacket;

// Packets from configuration (used in play state also with another ids)
pub use crate::states::configuration::s2c::{
    CookieRequestPacket, ResourcePackPushPacket, StoreCookiePacket,
};

use super::types;
use crate::connection::McPacket;
use crate::connection::s2c::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;
use bytes::Bytes;

#[derive(Debug)]
pub enum PlayState {
    ChangeDifficulty(ChangeDifficultyPacket),
    Commands(CommandsPacket),
    CookieRequest(CookieRequestPacket),
    Login(LoginPacket),
    PlayerInfoUpdate(PlayerInfoUpdatePacket),
    ResourcePackPush(ResourcePackPushPacket),
    StoreCookie(StoreCookiePacket),
    Another(AnotherPacket), // Other uninteresting packets
}

impl_clientbound_state! {
    state = "Play";
    enum PlayState;
    match protocol {
        5..=46 => {
            0x01 => Login: LoginPacket,
            0x38 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0..=0x40 => Another: AnotherPacket,
        },
        47..=106 => {
            0x01 => Login: LoginPacket,
            0x38 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x41 => ChangeDifficulty: ChangeDifficultyPacket,
            0x48 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x49 => Another: AnotherPacket,
        },
        107..=109 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x23 => Login: LoginPacket,
            0x2d => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x32 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x4c => Another: AnotherPacket,
        },
        110..=334 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x23 => Login: LoginPacket,
            0x2d => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x32 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x4b => Another: AnotherPacket,
        },
        335..=337 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x23 => Login: LoginPacket,
            0x2d => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x33 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x4e => Another: AnotherPacket,
        },
        338..=392 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x23 => Login: LoginPacket,
            0x2e => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x34 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x4f => Another: AnotherPacket,
        },
        393..=476 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x25 => Login: LoginPacket,
            0x30 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x37 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x55 => Another: AnotherPacket,
        },
        477..=497 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x25 => Login: LoginPacket,
            0x33 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x39 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x5b => Another: AnotherPacket,
        },
        498..=572 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x25 => Login: LoginPacket,
            0x33 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x39 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x5c => Another: AnotherPacket,
        },
        573..=734 => {
            0x0e => ChangeDifficulty: ChangeDifficultyPacket,
            0x12 => Commands: CommandsPacket,
            0x26 => Login: LoginPacket,
            0x34 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x3a => ResourcePackPush: ResourcePackPushPacket,
            0..=0x5c => Another: AnotherPacket,
        },
        735..=750 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x25 => Login: LoginPacket,
            0x33 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x39 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x5b => Another: AnotherPacket,
        },
        751..=754 => {
            0x0d => ChangeDifficulty: ChangeDifficultyPacket,
            0x10 => Commands: CommandsPacket,
            0x24 => Login: LoginPacket,
            0x32 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x38 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x5b => Another: AnotherPacket,
        },
        755..=756 => {
            0x0e => ChangeDifficulty: ChangeDifficultyPacket,
            0x12 => Commands: CommandsPacket,
            0x26 => Login: LoginPacket,
            0x36 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x3c => ResourcePackPush: ResourcePackPushPacket,
            0..=0x66 => Another: AnotherPacket,
        },
        757..=758 => {
            0x0e => ChangeDifficulty: ChangeDifficultyPacket,
            0x12 => Commands: CommandsPacket,
            0x26 => Login: LoginPacket,
            0x36 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x3c => ResourcePackPush: ResourcePackPushPacket,
            0..=0x67 => Another: AnotherPacket,
        },
        759 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x0f => Commands: CommandsPacket,
            0x23 => Login: LoginPacket,
            0x34 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x3a => ResourcePackPush: ResourcePackPushPacket,
            0..=0x68 => Another: AnotherPacket,
        },
        760 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x0f => Commands: CommandsPacket,
            0x25 => Login: LoginPacket,
            0x37 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x3d => ResourcePackPush: ResourcePackPushPacket,
            0..=0x6b => Another: AnotherPacket,
        },
        761 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x0e => Commands: CommandsPacket,
            0x24 => Login: LoginPacket,
            0x36 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x3c => ResourcePackPush: ResourcePackPushPacket,
            0..=0x6a => Another: AnotherPacket,
        },
        762..=763 => {
            0x0c => ChangeDifficulty: ChangeDifficultyPacket,
            0x10 => Commands: CommandsPacket,
            0x28 => Login: LoginPacket,
            0x3a => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x40 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x6e => Another: AnotherPacket,
        },
        764 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x29 => Login: LoginPacket,
            0x3c => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x42 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x70 => Another: AnotherPacket,
        },
        765 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x29 => Login: LoginPacket,
            0x3c => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x44 => ResourcePackPush: ResourcePackPushPacket,
            0..=0x74 => Another: AnotherPacket,
        },
        766 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x16 => CookieRequest: CookieRequestPacket,
            0x2b => Login: LoginPacket,
            0x3e => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x46 => ResourcePackPush: ResourcePackPushPacket,
            0x6b => StoreCookie: StoreCookiePacket,
            0..=0x79 => Another: AnotherPacket,
        },
        767 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x16 => CookieRequest: CookieRequestPacket,
            0x2b => Login: LoginPacket,
            0x3e => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x46 => ResourcePackPush: ResourcePackPushPacket,
            0x6b => StoreCookie: StoreCookiePacket,
            0..=0x7b => Another: AnotherPacket,
        },
        768..=769 => {
            0x0b => ChangeDifficulty: ChangeDifficultyPacket,
            0x11 => Commands: CommandsPacket,
            0x16 => CookieRequest: CookieRequestPacket,
            0x2c => Login: LoginPacket,
            0x40 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x4b => ResourcePackPush: ResourcePackPushPacket,
            0x72 => StoreCookie: StoreCookiePacket,
            0..=0x82 => Another: AnotherPacket,
        },
        770 => {
            0x0a => ChangeDifficulty: ChangeDifficultyPacket,
            0x10 => Commands: CommandsPacket,
            0x15 => CookieRequest: CookieRequestPacket,
            0x2b => Login: LoginPacket,
            0x3f => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x4a => ResourcePackPush: ResourcePackPushPacket,
            0x71 => StoreCookie: StoreCookiePacket,
            0..=0x82 => Another: AnotherPacket,
        },
        771..=772 => {
            0x0a => ChangeDifficulty: ChangeDifficultyPacket,
            0x10 => Commands: CommandsPacket,
            0x15 => CookieRequest: CookieRequestPacket,
            0x2b => Login: LoginPacket,
            0x3f => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x4a => ResourcePackPush: ResourcePackPushPacket,
            0x71 => StoreCookie: StoreCookiePacket,
            0..=0x85 => Another: AnotherPacket,
        },
        773..=774 => {
            0x0a => ChangeDifficulty: ChangeDifficultyPacket,
            0x10 => Commands: CommandsPacket,
            0x15 => CookieRequest: CookieRequestPacket,
            0x30 => Login: LoginPacket,
            0x44 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x4f => ResourcePackPush: ResourcePackPushPacket,
            0x76 => StoreCookie: StoreCookiePacket,
            0..=0x8a => Another: AnotherPacket,
        },
        775..=776 => {
            0x0a => ChangeDifficulty: ChangeDifficultyPacket,
            0x10 => Commands: CommandsPacket,
            0x15 => CookieRequest: CookieRequestPacket,
            0x31 => Login: LoginPacket,
            0x46 => PlayerInfoUpdate: PlayerInfoUpdatePacket,
            0x51 => ResourcePackPush: ResourcePackPushPacket,
            0x78 => StoreCookie: StoreCookiePacket,
            0..=0x8c => Another: AnotherPacket,
        },
    }
}

#[derive(Debug)]
pub struct AnotherPacket {}
impl ClientBoundPacket for AnotherPacket {
    const MC_NAME: &str = "";
    fn parse(_: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        Ok(Self {})
    }
}
