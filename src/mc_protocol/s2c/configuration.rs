use super::{ClientBoundPacket, ClientBoundState, PacketError, ParsePacketError, try_split_to};
use crate::impl_clientbound_state;
use crate::mc_protocol::McPacket;
use crate::mc_protocol::types::bool::McBool;
use crate::mc_protocol::types::json_text_component::{JsonTextComponent, MCJsonField};
use crate::mc_protocol::types::mc_string::McStringField;
use crate::mc_protocol::types::varint::McVarInt;
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug)]
pub enum PacketEnum {
    CookieRequest(CookieRequestPacket),
    PluginMessage(PluginMessagePacket),
    Disconnect(DisconnectPacket),
    FinishConfiguration(FinishConfigurationPacket),
    KeepAlive(KeepAlivePacket),
    Ping(PingPacket),
    ResetChat(ResetChatPacket),
    RegistryData(RegistryDataPacket),
    RemoveResourcePack(RemoveResourcePackPacket),
    AddResourcePack(AddResourcePackPacket),
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

impl_clientbound_state!(
    PacketEnum, "Configuration",
    CookieRequest => CookieRequestPacket,
    PluginMessage => PluginMessagePacket,
    Disconnect => DisconnectPacket,
    FinishConfiguration => FinishConfigurationPacket,
    KeepAlive => KeepAlivePacket,
    Ping => PingPacket,
    ResetChat => ResetChatPacket,
    RegistryData => RegistryDataPacket,
    RemoveResourcePack => RemoveResourcePackPacket,
    AddResourcePack => AddResourcePackPacket,
    StoreCookie => StoreCookiePacket,
    Transfer => TransferPacket,
    FeatureFlags => FeatureFlagsPacket,
    UpdateTags => UpdateTagsPacket,
    KnownPacks => KnownPacksPacket,
    CustomReportDetails => CustomReportDetailsPacket,
    ServerLinks => ServerLinksPacket,
    ClearDialog => ClearDialogPacket,
    ShowDialog => ShowDialogPacket,
    CodeOfConduct => CodeOfConductPacket,
);

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request
#[derive(Debug)]
pub struct CookieRequestPacket {
    pub key: String,
}
impl ClientBoundPacket for CookieRequestPacket {
    const ID: i32 = 0x00;
    const MC_NAME: &str = "cookie_request";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let key = McStringField(32767).read_from_buf(&mut data)?;
        Ok(Self { key })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound)
#[derive(Debug)]
pub struct PluginMessagePacket {
    pub channel: String,
    pub data: Bytes,
}
impl ClientBoundPacket for PluginMessagePacket {
    const ID: i32 = 0x01;
    const MC_NAME: &str = "custom_payload";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let channel = McStringField(32767).read_from_buf(&mut data)?;
        Ok(Self { channel, data })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect
#[derive(Debug)]
pub struct DisconnectPacket {
    pub reason: JsonTextComponent,
}
impl ClientBoundPacket for DisconnectPacket {
    const ID: i32 = 0x02;
    const MC_NAME: &str = "disconnect";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let reason = MCJsonField(262144).read_from_buf(&mut data)?;
        Ok(Self { reason })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Finish_Configuration
#[derive(Debug)]
pub struct FinishConfigurationPacket {}
impl ClientBoundPacket for FinishConfigurationPacket {
    const ID: i32 = 0x03;
    const MC_NAME: &str = "finish_configuration";
    fn parse(_data: Bytes) -> Result<Self, ParsePacketError> {
        Ok(Self {})
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound)
#[derive(Debug)]
pub struct KeepAlivePacket {
    pub keep_alive_id: i64,
}
impl ClientBoundPacket for KeepAlivePacket {
    const ID: i32 = 0x04;
    const MC_NAME: &str = "keep_alive";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let keep_alive_id = data.try_get_i64()?;
        Ok(Self { keep_alive_id })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping
#[derive(Debug)]
pub struct PingPacket {
    pub id: i32,
}
impl ClientBoundPacket for PingPacket {
    const ID: i32 = 0x05;
    const MC_NAME: &str = "ping";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let id = data.try_get_i32()?;
        Ok(Self { id })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Reset_Chat
#[derive(Debug)]
pub struct ResetChatPacket {}
impl ClientBoundPacket for ResetChatPacket {
    const ID: i32 = 0x06;
    const MC_NAME: &str = "reset_chat";
    fn parse(_data: Bytes) -> Result<Self, ParsePacketError> {
        Ok(Self {})
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Registry_Data
#[derive(Debug)]
pub struct RegistryDataPacket {
    pub registry_id: String,
    pub entries_data: Bytes,
}
impl ClientBoundPacket for RegistryDataPacket {
    const ID: i32 = 0x07;
    const MC_NAME: &str = "registry_data";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let registry_id = McStringField(32767).read_from_buf(&mut data)?;
        Ok(Self {
            registry_id,
            entries_data: data,
        })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Resource_Pack
#[derive(Debug)]
pub struct RemoveResourcePackPacket {
    pub uuid: Option<Uuid>,
}
impl ClientBoundPacket for RemoveResourcePackPacket {
    const ID: i32 = 0x08;
    const MC_NAME: &str = "resource_pack_pop";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let has_uuid = McBool::read_from_buf(&mut data)?.0;
        let uuid = if has_uuid {
            Some(Uuid::from_u128(data.try_get_u128()?))
        } else {
            None
        };
        Ok(Self { uuid })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Add_Resource_Pack
#[derive(Debug)]
pub struct AddResourcePackPacket {
    pub uuid: Uuid,
    pub url: String,
    pub hash: String,
    pub forced: bool,
    pub prompt_message: Option<JsonTextComponent>,
}
impl ClientBoundPacket for AddResourcePackPacket {
    const ID: i32 = 0x09;
    const MC_NAME: &str = "resource_pack_push";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let uuid = Uuid::from_u128(data.try_get_u128()?);
        let url = McStringField(32767).read_from_buf(&mut data)?;
        let hash = McStringField(40).read_from_buf(&mut data)?;
        let forced = McBool::read_from_buf(&mut data)?.0;

        let has_prompt = McBool::read_from_buf(&mut data)?.0;
        let prompt_message = if has_prompt {
            Some(MCJsonField(262144).read_from_buf(&mut data)?)
        } else {
            None
        };

        Ok(Self {
            uuid,
            url,
            hash,
            forced,
            prompt_message,
        })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Store_Cookie
#[derive(Debug)]
pub struct StoreCookiePacket {
    pub key: String,
    pub payload: Bytes,
}
impl ClientBoundPacket for StoreCookiePacket {
    const ID: i32 = 0x0A;
    const MC_NAME: &str = "store_cookie";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let key = McStringField(32767).read_from_buf(&mut data)?;
        let payload_len = McVarInt::read_from_buf(&mut data)?.with_check(0, 5120)?.0 as usize;
        let payload = try_split_to(&mut data, payload_len)?;
        Ok(Self { key, payload })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Transfer
#[derive(Debug)]
pub struct TransferPacket {
    pub host: String,
    pub port: i32,
}
impl ClientBoundPacket for TransferPacket {
    const ID: i32 = 0x0B;
    const MC_NAME: &str = "transfer";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let host = McStringField(32767).read_from_buf(&mut data)?;
        let port = McVarInt::read_from_buf(&mut data)?.0;
        Ok(Self { host, port })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Feature_Flags
#[derive(Debug)]
pub struct FeatureFlagsPacket {
    pub features: Vec<String>,
}
impl ClientBoundPacket for FeatureFlagsPacket {
    const ID: i32 = 0x0C;
    const MC_NAME: &str = "update_enabled_features";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        const MAX_FEATURES: i32 = 1024; // Minecraft client doesn't have limit for features
        let count = McVarInt::read_from_buf(&mut data)?
            .with_check(0, MAX_FEATURES)?
            .0 as usize;
        let mut features = Vec::with_capacity(count);
        for _ in 0..count {
            features.push(McStringField(32767).read_from_buf(&mut data)?);
        }
        Ok(Self { features })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Tags
#[derive(Debug)]
pub struct UpdateTagsPacket {
    pub data: Bytes,
}
impl ClientBoundPacket for UpdateTagsPacket {
    const ID: i32 = 0x0D;
    const MC_NAME: &str = "update_tags";
    fn parse(data: Bytes) -> Result<Self, ParsePacketError> {
        Ok(Self { data })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(clientbound)
#[derive(Debug)]
pub struct KnownPacksPacket {
    pub known_packs: Vec<KnownPack>,
}

impl ClientBoundPacket for KnownPacksPacket {
    const ID: i32 = 0x0E;
    const MC_NAME: &str = "select_known_packs";

    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        const MAX_PACKS: i32 = 32; // Minecraft doesn't set limit for packs limit
        let count = McVarInt::read_from_buf(&mut data)?
            .with_check(0, MAX_PACKS)?
            .0 as usize;
        let mut known_packs = Vec::with_capacity(count);
        for _ in 0..count {
            known_packs.push(KnownPack::read_from_buf(&mut data)?);
        }
        Ok(Self { known_packs })
    }
}
#[derive(Debug)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl KnownPack {
    fn read_from_buf(data: &mut Bytes) -> Result<Self, ParsePacketError> {
        let namespace = McStringField(32767).read_from_buf(data)?;
        let id = McStringField(32767).read_from_buf(data)?;
        let version = McStringField(32767).read_from_buf(data)?;
        Ok(Self {
            namespace,
            id,
            version,
        })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Report_Details
#[derive(Debug)]
pub struct CustomReportDetailsPacket {
    pub details: Vec<ReportDetail>,
}

impl ClientBoundPacket for CustomReportDetailsPacket {
    const ID: i32 = 0x0F;
    const MC_NAME: &str = "custom_report_details";

    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        const MAX_DETAILS: i32 = 1024; // Minecraft client doesn't have limit for report details

        let count = McVarInt::read_from_buf(&mut data)?
            .with_check(0, MAX_DETAILS)?
            .0 as usize;
        let mut details = Vec::with_capacity(count);
        for _ in 0..count {
            details.push(ReportDetail::read_from_buf(&mut data)?);
        }
        Ok(Self { details })
    }
}
#[derive(Debug)]
pub struct ReportDetail {
    pub title: String,
    pub description: String,
}

impl ReportDetail {
    fn read_from_buf(data: &mut Bytes) -> Result<Self, ParsePacketError> {
        let title = McStringField(128).read_from_buf(data)?;
        let description = McStringField(4096).read_from_buf(data)?;
        Ok(Self { title, description })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Server_Links
#[derive(Debug)]
pub struct ServerLinksPacket {
    pub links: Vec<ServerLink>,
}

impl ClientBoundPacket for ServerLinksPacket {
    const ID: i32 = 0x10;
    const MC_NAME: &str = "server_links";

    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        const MAX_LINKS: i32 = 16; // Minecraft doesn't set limit for link's count
        let count = McVarInt::read_from_buf(&mut data)?
            .with_check(0, MAX_LINKS)?
            .0 as usize;
        let mut links = Vec::with_capacity(count);
        for _ in 0..count {
            links.push(ServerLink::read_from_buf(&mut data)?);
        }
        Ok(Self { links })
    }
}
#[derive(Debug)]
pub enum ServerLinkLabel {
    BuiltIn(i32),
    Custom(JsonTextComponent),
}

#[derive(Debug)]
pub struct ServerLink {
    pub label: ServerLinkLabel,
    pub url: String,
}
impl ServerLink {
    fn read_from_buf(data: &mut Bytes) -> Result<Self, ParsePacketError> {
        let is_built_in = McBool::read_from_buf(data)?.0;
        let label = match is_built_in {
            true => {
                let enum_id = McVarInt::read_from_buf(data)?.0;
                ServerLinkLabel::BuiltIn(enum_id)
            }
            false => {
                let text = MCJsonField(262144).read_from_buf(data)?;
                ServerLinkLabel::Custom(text)
            }
        };
        let url = McStringField(32767).read_from_buf(data)?;
        Ok(Self { label, url })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clear_Dialog
#[derive(Debug)]
pub struct ClearDialogPacket {}
impl ClientBoundPacket for ClearDialogPacket {
    const ID: i32 = 0x11;
    const MC_NAME: &str = "clear_dialog";
    fn parse(_data: Bytes) -> Result<Self, ParsePacketError> {
        Ok(Self {})
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Show_Dialog
#[derive(Debug)]
pub struct ShowDialogPacket {
    pub nbt_data: Bytes,
}
impl ClientBoundPacket for ShowDialogPacket {
    const ID: i32 = 0x12;
    const MC_NAME: &str = "show_dialog";
    fn parse(data: Bytes) -> Result<Self, ParsePacketError> {
        Ok(Self { nbt_data: data })
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Code_of_Conduct
#[derive(Debug)]
pub struct CodeOfConductPacket {
    pub code_of_conduct: String,
}
impl ClientBoundPacket for CodeOfConductPacket {
    const ID: i32 = 0x13;
    const MC_NAME: &str = "code_of_conduct";
    fn parse(mut data: Bytes) -> Result<Self, ParsePacketError> {
        let code_of_conduct = McStringField(32767).read_from_buf(&mut data)?;
        Ok(Self { code_of_conduct })
    }
}
