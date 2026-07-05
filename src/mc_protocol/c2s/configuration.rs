use super::ServerBoundPacket;
use crate::mc_protocol::types::mc_string::McStringField;
use crate::mc_protocol::types::varint::McVarInt;
use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information
pub struct ClientInformationPacket {
    pub locale: String,
    pub view_distance: u8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: i32,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
    pub particle_status: i32,
}
impl Default for ClientInformationPacket {
    fn default() -> Self {
        Self {
            locale: String::from("en_US"),
            view_distance: 8,
            chat_mode: 0,
            chat_colors: true,
            displayed_skin_parts: 0x7F,
            main_hand: 1,
            enable_text_filtering: false,
            allow_server_listings: true,
            particle_status: 0,
        }
    }
}
impl ServerBoundPacket for ClientInformationPacket {
    const ID: i32 = 0x00;
    const MC_NAME: &'static str = "client_information";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McStringField::write_to_buf(&self.locale, buf);
        buf.put_u8(self.view_distance);
        McVarInt(self.chat_mode).write_to_buf(buf);
        buf.put_u8(if self.chat_colors { 1 } else { 0 });
        buf.put_u8(self.displayed_skin_parts);
        McVarInt(self.main_hand).write_to_buf(buf);
        buf.put_u8(if self.enable_text_filtering { 1 } else { 0 });
        buf.put_u8(if self.allow_server_listings { 1 } else { 0 });
        McVarInt(self.particle_status).write_to_buf(buf);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Response
pub struct CookieResponsePacket<T>
where
    T: IntoIterator<Item = u8>,
{
    pub key: String,
    pub payload: Option<T>,
}
impl CookieResponsePacket<std::iter::Empty<u8>> {
    pub fn empty_payload(key: String) -> Self {
        Self { key, payload: None }
    }
}
impl<T> ServerBoundPacket for CookieResponsePacket<T>
where
    T: IntoIterator<Item = u8>,
{
    const ID: i32 = 0x01;
    const MC_NAME: &'static str = "cookie_response";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McStringField::write_to_buf(&self.key, buf);
        match self.payload {
            None => {
                buf.put_u8(0);
            }
            Some(payload) => {
                buf.put_u8(1);
                buf.extend(payload);
            }
        }
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(serverbound)
pub struct PluginMessagePacket {
    pub channel: String,
    pub data: Bytes,
}
impl ServerBoundPacket for PluginMessagePacket {
    const ID: i32 = 0x02;
    const MC_NAME: &'static str = "custom_payload";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McStringField::write_to_buf(&self.channel, buf);
        buf.extend(self.data);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Finish_Configuration
pub struct AcknowledgeFinishConfigurationPacket;
impl ServerBoundPacket for AcknowledgeFinishConfigurationPacket {
    const ID: i32 = 0x03;
    const MC_NAME: &'static str = "finish_configuration";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(serverbound)
pub struct KeepAlivePacket {
    pub keep_alive_id: i64,
}
impl ServerBoundPacket for KeepAlivePacket {
    const ID: i32 = 0x04;
    const MC_NAME: &'static str = "keep_alive";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        buf.put_i64(self.keep_alive_id);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong
pub struct PongPacket {
    pub id: i32,
}
impl ServerBoundPacket for PongPacket {
    const ID: i32 = 0x05;
    const MC_NAME: &'static str = "pong";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        buf.put_i32(self.id);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resource_Pack_Response
pub struct ResourcePackResponsePacket {
    pub uuid: Uuid,
    pub result: i32,
}
impl ResourcePackResponsePacket {
    pub fn accepted(uuid: Uuid) -> Self {
        Self { uuid, result: 3 }
    }
}
impl ServerBoundPacket for ResourcePackResponsePacket {
    const ID: i32 = 0x06;
    const MC_NAME: &'static str = "resource_pack";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        buf.extend_from_slice(self.uuid.as_bytes());
        McVarInt(self.result).write_to_buf(buf);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(serverbound)
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}
pub struct KnownPacksPacket {
    pub known_packs: Vec<KnownPack>,
}
impl ServerBoundPacket for KnownPacksPacket {
    const ID: i32 = 0x07;
    const MC_NAME: &'static str = "select_known_packs";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McVarInt(self.known_packs.len() as i32).write_to_buf(buf);
        for pack in self.known_packs {
            McStringField::write_to_buf(&pack.namespace, buf);
            McStringField::write_to_buf(&pack.id, buf);
            McStringField::write_to_buf(&pack.version, buf);
        }
    }
}
impl KnownPacksPacket {
    pub fn default(version: String) -> Self {
        Self {
            known_packs: vec![KnownPack {
                namespace: String::from("minecraft"),
                id: String::from("core"),
                version,
            }],
        }
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Click_Action
pub struct CustomClickActionPacket {
    pub id: String,
    pub payload: Bytes,
}
impl ServerBoundPacket for CustomClickActionPacket {
    const ID: i32 = 0x08;
    const MC_NAME: &'static str = "custom_click_action";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
        McStringField::write_to_buf(&self.id, buf);
        buf.extend(self.payload);
    }
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Accept_Code_of_Conduct
pub struct AcceptCodeOfConductPacket;
impl ServerBoundPacket for AcceptCodeOfConductPacket {
    const ID: i32 = 0x09;
    const MC_NAME: &'static str = "accept_code_of_conduct";
    fn encode(self, buf: &mut BytesMut) {
        McVarInt(Self::ID).write_to_buf(buf);
    }
}
