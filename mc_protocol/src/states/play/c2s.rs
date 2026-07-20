use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use crate::impl_serverbound_state;
pub use crate::states::configuration::c2s::{
    CustomPayloadResponsePacket, KeepAliveResponsePacket, PongPacket, ResourcePackResponsePacket,
};
pub use crate::states::login::c2s::CookieResponsePacket;

#[derive(Debug)]
pub enum C2SPlayState {
    CookieResponse(CookieResponsePacket),
    CustomPayloadResponse(CustomPayloadResponsePacket),
    KeepAliveResponse(KeepAliveResponsePacket),
    Pong(PongPacket),
    ResourcePackResponse(ResourcePackResponsePacket),
}

impl_serverbound_state! {
    state = "play";
    enum C2SPlayState;
    match protocol {
        5..=46 => {
            0x00 => KeepAliveResponse: KeepAliveResponsePacket,
            0x17 => CustomPayloadResponse: CustomPayloadResponsePacket,
        },
        47..=106 => {
            0x00 => KeepAliveResponse: KeepAliveResponsePacket,
            0x17 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x19 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        107..=334 => {
            0x09 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x0b => KeepAliveResponse: KeepAliveResponsePacket,
            0x16 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        335..=337 => {
            0x0a => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x0c => KeepAliveResponse: KeepAliveResponsePacket,
            0x18 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        338..=392 => {
            0x09 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x0b => KeepAliveResponse: KeepAliveResponsePacket,
            0x18 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        393..=476 => {
            0x0a => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x0e => KeepAliveResponse: KeepAliveResponsePacket,
            0x1d => ResourcePackResponse: ResourcePackResponsePacket,
        },
        477..=734 => {
            0x0b => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x0f => KeepAliveResponse: KeepAliveResponsePacket,
            0x1f => ResourcePackResponse: ResourcePackResponsePacket,
        },
        735..=750 => {
            0x0b => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x10 => KeepAliveResponse: KeepAliveResponsePacket,
            0x20 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        751..=754 => {
            0x0b => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x10 => KeepAliveResponse: KeepAliveResponsePacket,
            0x21 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        755..=758 => {
            0x0a => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x0f => KeepAliveResponse: KeepAliveResponsePacket,
            0x1d => Pong: PongPacket,
            0x21 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        759 => {
            0x0c => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x11 => KeepAliveResponse: KeepAliveResponsePacket,
            0x1f => Pong: PongPacket,
            0x23 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        760 => {
            0x0d => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x12 => KeepAliveResponse: KeepAliveResponsePacket,
            0x20 => Pong: PongPacket,
            0x24 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        761 => {
            0x0c => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x11 => KeepAliveResponse: KeepAliveResponsePacket,
            0x1f => Pong: PongPacket,
            0x24 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        762..=763 => {
            0x0d => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x12 => KeepAliveResponse: KeepAliveResponsePacket,
            0x20 => Pong: PongPacket,
            0x24 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        764 => {
            0x0f => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x14 => KeepAliveResponse: KeepAliveResponsePacket,
            0x23 => Pong: PongPacket,
            0x27 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        765 => {
            0x10 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x15 => KeepAliveResponse: KeepAliveResponsePacket,
            0x24 => Pong: PongPacket,
            0x28 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        766..=767 => {
            0x11 => CookieResponse: CookieResponsePacket,
            0x12 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x18 => KeepAliveResponse: KeepAliveResponsePacket,
            0x27 => Pong: PongPacket,
            0x2b => ResourcePackResponse: ResourcePackResponsePacket,
        },
        768 => {
            0x13 => CookieResponse: CookieResponsePacket,
            0x14 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x1a => KeepAliveResponse: KeepAliveResponsePacket,
            0x29 => Pong: PongPacket,
            0x2d => ResourcePackResponse: ResourcePackResponsePacket,
        },
        769..=770 => {
            0x13 => CookieResponse: CookieResponsePacket,
            0x14 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x1a => KeepAliveResponse: KeepAliveResponsePacket,
            0x2b => Pong: PongPacket,
            0x2f => ResourcePackResponse: ResourcePackResponsePacket,
        },
        771..=774 => {
            0x14 => CookieResponse: CookieResponsePacket,
            0x15 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x1b => KeepAliveResponse: KeepAliveResponsePacket,
            0x2c => Pong: PongPacket,
            0x30 => ResourcePackResponse: ResourcePackResponsePacket,
        },
        775..=776 => {
            0x15 => CookieResponse: CookieResponsePacket,
            0x16 => CustomPayloadResponse: CustomPayloadResponsePacket,
            0x1c => KeepAliveResponse: KeepAliveResponsePacket,
            0x2d => Pong: PongPacket,
            0x31 => ResourcePackResponse: ResourcePackResponsePacket,
        },
    }
}
