mod clear_dialog_packet;
pub use clear_dialog_packet::ClearDialogPacket;
mod code_of_conduct_packet;
pub use code_of_conduct_packet::CodeOfConductPacket;
mod store_cookie;
pub use store_cookie::StoreCookiePacket;
pub mod custom_report_details_packet;
pub use custom_report_details_packet::CustomReportDetailsPacket;
mod disconnect_packet;
pub use disconnect_packet::DisconnectPacket;
mod feature_flags_packet;
pub use feature_flags_packet::FeatureFlagsPacket;
mod finish_configuration_packet;
pub use finish_configuration_packet::FinishConfigurationPacket;
mod keep_alive_packet;
pub use keep_alive_packet::KeepAlivePacket;
mod ping_packet;
pub use ping_packet::PingPacket;
mod plugin_message_packet;
pub use plugin_message_packet::PluginMessagePacket;
mod registry_data_packet;
pub use registry_data_packet::RegistryDataPacket;
mod reset_chat_packet;
pub use reset_chat_packet::ResetChatPacket;
pub mod select_known_pack;
pub use select_known_pack::KnownPacksPacket;
pub mod resource_pack_push;
pub use resource_pack_push::ResourcePackPushPacket;
pub mod resource_pack_pop;
pub use resource_pack_pop::RemoveResourcePackPacket;
pub mod server_links;
pub use server_links::ServerLinksPacket;
mod show_dialog_packet;
pub use show_dialog_packet::ShowDialogPacket;
mod update_tags_packet;
pub use update_tags_packet::UpdateTagsPacket;
mod transfer_packet;
pub use transfer_packet::TransferPacket;

use super::types;
use crate::connection::McPacket;
use crate::connection::s2c::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError};
use crate::impl_clientbound_state;
pub use crate::states::login::s2c::CookieRequestPacket;

#[derive(Debug)]
pub enum ConfigurationState {
    CookieRequest(CookieRequestPacket),
    PluginMessage(PluginMessagePacket),
    Disconnect(DisconnectPacket),
    FinishConfiguration(FinishConfigurationPacket),
    KeepAlive(KeepAlivePacket),
    Ping(PingPacket),
    ResetChat(ResetChatPacket),
    RegistryData(RegistryDataPacket),
    RemoveResourcePack(RemoveResourcePackPacket),
    AddResourcePack(ResourcePackPushPacket),
    StoreCookie(StoreCookiePacket),
    Transfer(TransferPacket),
    FeatureFlags(FeatureFlagsPacket),
    UpdateTags(UpdateTagsPacket),
    KnownPacks(KnownPacksPacket),
    CustomReportDetails(CustomReportDetailsPacket),
    ServerLinks(ServerLinksPacket),
    ClearDialog(ClearDialogPacket),
    ShowDialog(ShowDialogPacket),
    CodeOfConduct(CodeOfConductPacket),
}

impl_clientbound_state! {
    state = "Configuration";
    enum ConfigurationState;
    match protocol {
        764  => {
            0x00 => PluginMessage: PluginMessagePacket,
            0x01 => Disconnect: DisconnectPacket,
            0x02 => FinishConfiguration: FinishConfigurationPacket,
            0x03 => KeepAlive: KeepAlivePacket,
            0x04 => Ping: PingPacket,
            0x05 => RegistryData: RegistryDataPacket,
            0x06 => AddResourcePack: ResourcePackPushPacket,
            0x07 => FeatureFlags: FeatureFlagsPacket,
            0x08 => UpdateTags: UpdateTagsPacket,
        },
        765  => {
            0x00 => PluginMessage: PluginMessagePacket,
            0x01 => Disconnect: DisconnectPacket,
            0x02 => FinishConfiguration: FinishConfigurationPacket,
            0x03 => KeepAlive: KeepAlivePacket,
            0x04 => Ping: PingPacket,
            0x05 => RegistryData: RegistryDataPacket,
            0x06 => RemoveResourcePack: RemoveResourcePackPacket,
            0x07 => AddResourcePack: ResourcePackPushPacket,
            0x08 => FeatureFlags: FeatureFlagsPacket,
            0x09 => UpdateTags: UpdateTagsPacket,
        },
        766  => {
            0x00 => CookieRequest: CookieRequestPacket,
            0x01 => PluginMessage: PluginMessagePacket,
            0x02 => Disconnect: DisconnectPacket,
            0x03 => FinishConfiguration: FinishConfigurationPacket,
            0x04 => KeepAlive: KeepAlivePacket,
            0x05 => Ping: PingPacket,
            0x06 => ResetChat: ResetChatPacket,
            0x07 => RegistryData: RegistryDataPacket,
            0x08 => RemoveResourcePack: RemoveResourcePackPacket,
            0x09 => AddResourcePack: ResourcePackPushPacket,
            0x0a => StoreCookie: StoreCookiePacket,
            0x0b => Transfer: TransferPacket,
            0x0c => FeatureFlags: FeatureFlagsPacket,
            0x0d => UpdateTags: UpdateTagsPacket,
            0x0e => KnownPacks: KnownPacksPacket,
        },
        767..=770  => {
            0x00 => CookieRequest: CookieRequestPacket,
            0x01 => PluginMessage: PluginMessagePacket,
            0x02 => Disconnect: DisconnectPacket,
            0x03 => FinishConfiguration: FinishConfigurationPacket,
            0x04 => KeepAlive: KeepAlivePacket,
            0x05 => Ping: PingPacket,
            0x06 => ResetChat: ResetChatPacket,
            0x07 => RegistryData: RegistryDataPacket,
            0x08 => RemoveResourcePack: RemoveResourcePackPacket,
            0x09 => AddResourcePack: ResourcePackPushPacket,
            0x0a => StoreCookie: StoreCookiePacket,
            0x0b => Transfer: TransferPacket,
            0x0c => FeatureFlags: FeatureFlagsPacket,
            0x0d => UpdateTags: UpdateTagsPacket,
            0x0e => KnownPacks: KnownPacksPacket,
            0x0f => CustomReportDetails: CustomReportDetailsPacket,
            0x10 => ServerLinks: ServerLinksPacket,
        },
        771..=772  => {
            0x00 => CookieRequest: CookieRequestPacket,
            0x01 => PluginMessage: PluginMessagePacket,
            0x02 => Disconnect: DisconnectPacket,
            0x03 => FinishConfiguration: FinishConfigurationPacket,
            0x04 => KeepAlive: KeepAlivePacket,
            0x05 => Ping: PingPacket,
            0x06 => ResetChat: ResetChatPacket,
            0x07 => RegistryData: RegistryDataPacket,
            0x08 => RemoveResourcePack: RemoveResourcePackPacket,
            0x09 => AddResourcePack: ResourcePackPushPacket,
            0x0a => StoreCookie: StoreCookiePacket,
            0x0b => Transfer: TransferPacket,
            0x0c => FeatureFlags: FeatureFlagsPacket,
            0x0d => UpdateTags: UpdateTagsPacket,
            0x0e => KnownPacks: KnownPacksPacket,
            0x0f => CustomReportDetails: CustomReportDetailsPacket,
            0x10 => ServerLinks: ServerLinksPacket,
            0x11 => ClearDialog: ClearDialogPacket,
            0x12 => ShowDialog: ShowDialogPacket,
        },
        773..=776  => {
            0x00 => CookieRequest: CookieRequestPacket,
            0x01 => PluginMessage : PluginMessagePacket,
            0x02 => Disconnect: DisconnectPacket,
            0x03 => FinishConfiguration: FinishConfigurationPacket,
            0x04 => KeepAlive: KeepAlivePacket,
            0x05 => Ping: PingPacket,
            0x06 => ResetChat: ResetChatPacket,
            0x07 => RegistryData: RegistryDataPacket,
            0x08 => RemoveResourcePack: RemoveResourcePackPacket,
            0x09 => AddResourcePack: ResourcePackPushPacket,
            0x0a => StoreCookie: StoreCookiePacket,
            0x0b => Transfer: TransferPacket,
            0x0c => FeatureFlags: FeatureFlagsPacket,
            0x0d => UpdateTags: UpdateTagsPacket,
            0x0e => KnownPacks: KnownPacksPacket,
            0x0f => CustomReportDetails: CustomReportDetailsPacket,
            0x10 => ServerLinks: ServerLinksPacket,
            0x11 => ClearDialog: ClearDialogPacket,
            0x12 => ShowDialog: ShowDialogPacket,
            0x13 => CodeOfConduct: CodeOfConductPacket,
        },
    }
}
