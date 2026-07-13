mod accept_code_of_conduct;
pub use accept_code_of_conduct::AcceptCodeOfConductPacket;
mod custom_click_action_packet;
pub use custom_click_action_packet::CustomClickActionPacket;
mod custom_payload;
pub use custom_payload::CustomPayloadPacket;
mod finish_configuration_packet;
pub use finish_configuration_packet::FinishConfigurationPacket;
mod keep_alive_packet;
pub use keep_alive_packet::KeepAlivePacket;
mod pong_packet;
pub use pong_packet::PongPacket;
mod select_known_packs;
pub use select_known_packs::SelectKnownPacksPacket;
mod resource_pack_response;
pub use resource_pack_response::ResourcePackResponsePacket;
mod client_information_packet;

pub use client_information_packet::ClientInformationPacket;

use crate::connection::c2s::{ServerBoundPacket, ServerBoundState};
use super::types;
use crate::impl_serverbound_state;
pub use crate::states::login::c2s::CookieResponsePacket;

#[derive(Debug)]
pub enum ConfigurationState {
    ClientInformation(ClientInformationPacket),
    CookieResponse(CookieResponsePacket),
    CustomPayload(CustomPayloadPacket),
    FinishConfiguration(FinishConfigurationPacket),
    KeepAlive(KeepAlivePacket),
    Pong(PongPacket),
    ResourcePackResponse(ResourcePackResponsePacket),
    SelectKnownPacks(SelectKnownPacksPacket),
    CustomClickAction(CustomClickActionPacket),
    AcceptCodeOfConduct(AcceptCodeOfConductPacket),
}

impl_serverbound_state! {
    state = "configuration";
    enum ConfigurationState;
        match protocol {
            764..=765 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CustomPayload: CustomPayloadPacket,
                0x02 => FinishConfiguration: FinishConfigurationPacket,
                0x03 => KeepAlive: KeepAlivePacket,
                0x04 => Pong: PongPacket,
                0x05 => ResourcePackResponse: ResourcePackResponsePacket,
            },
            766 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayload: CustomPayloadPacket,
                0x03 => FinishConfiguration: FinishConfigurationPacket,
                0x04 => KeepAlive: KeepAlivePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
            },
            767..=770 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayload: CustomPayloadPacket,
                0x03 => FinishConfiguration: FinishConfigurationPacket,
                0x04 => KeepAlive: KeepAlivePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
            },
            771..=772 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayload: CustomPayloadPacket,
                0x03 => FinishConfiguration: FinishConfigurationPacket,
                0x04 => KeepAlive: KeepAlivePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
                0x08 => CustomClickAction: CustomClickActionPacket,
            },
            773..=776 => {
                0x00 => ClientInformation: ClientInformationPacket,
                0x01 => CookieResponse: CookieResponsePacket,
                0x02 => CustomPayload: CustomPayloadPacket,
                0x03 => FinishConfiguration: FinishConfigurationPacket,
                0x04 => KeepAlive: KeepAlivePacket,
                0x05 => Pong: PongPacket,
                0x06 => ResourcePackResponse: ResourcePackResponsePacket,
                0x07 => SelectKnownPacks: SelectKnownPacksPacket,
                0x08 => CustomClickAction: CustomClickActionPacket,
                0x09 => AcceptCodeOfConduct: AcceptCodeOfConductPacket,
            },
    }
}
