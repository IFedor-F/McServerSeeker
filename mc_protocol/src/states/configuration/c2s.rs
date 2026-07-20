mod accept_code_of_conduct;
pub use accept_code_of_conduct::AcceptCodeOfConductPacket;
mod custom_click_action;
pub use custom_click_action::CustomClickActionPacket;
mod custom_payload;
pub use custom_payload::CustomPayloadResponsePacket;
mod finish_configuration;
pub use finish_configuration::AckFinishConfigurationPacket;
mod keep_alive;
pub use keep_alive::KeepAliveResponsePacket;
mod pong;
pub use pong::PongPacket;
mod select_known_packs;
pub use select_known_packs::SelectKnownPacksPacket;
mod resource_pack_response;
pub use resource_pack_response::ResourcePackResponsePacket;
mod client_information;

pub use client_information::ClientInformationPacket;

use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use super::types;
use crate::impl_serverbound_state;
pub use crate::states::login::c2s::CookieResponsePacket;

#[derive(Debug)]
pub enum C2SConfigurationState {
    ClientInformation(ClientInformationPacket),
    CookieResponse(CookieResponsePacket),
    CustomPayloadResponse(CustomPayloadResponsePacket),
    AckFinishConfiguration(AckFinishConfigurationPacket),
    KeepAliveResponse(KeepAliveResponsePacket),
    Pong(PongPacket),
    ResourcePackResponse(ResourcePackResponsePacket),
    SelectKnownPacks(SelectKnownPacksPacket),
    CustomClickAction(CustomClickActionPacket),
    AcceptCodeOfConduct(AcceptCodeOfConductPacket),
}

impl_serverbound_state! {
    state = "configuration";
    enum C2SConfigurationState;
        match protocol {
            764..=765 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CustomPayloadResponse: CustomPayloadResponsePacket,
                0x02 => AckFinishConfiguration: AckFinishConfigurationPacket,
                0x03 => KeepAliveResponse: KeepAliveResponsePacket,
                0x04 => Pong: PongPacket,
                0x05 => ResourcePackResponse: ResourcePackResponsePacket,
            },
            766..=770 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayloadResponse: CustomPayloadResponsePacket,
                0x03 => AckFinishConfiguration: AckFinishConfigurationPacket,
                0x04 => KeepAliveResponse: KeepAliveResponsePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
            },
            771..=772 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayloadResponse: CustomPayloadResponsePacket,
                0x03 => AckFinishConfiguration: AckFinishConfigurationPacket,
                0x04 => KeepAliveResponse: KeepAliveResponsePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
                0x08 => CustomClickAction: CustomClickActionPacket,
            },
            773..=776 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayloadResponse: CustomPayloadResponsePacket,
                0x03 => AckFinishConfiguration: AckFinishConfigurationPacket,
                0x04 => KeepAliveResponse: KeepAliveResponsePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
                0x08 => CustomClickAction: CustomClickActionPacket,
                0x09 => AcceptCodeOfConduct: AcceptCodeOfConductPacket,
            },
    }
}
