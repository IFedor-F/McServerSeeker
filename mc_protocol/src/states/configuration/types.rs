use crate::connection::s2c::ParsePacketError;
use crate::states::configuration::s2c::ResourcePackPushPacket;
use crate::types::{
    McBool, McModernNbtField, McNbtTag, McPrefixedArrayField, McReadBuf, McStringField,
    McStringFieldError, McVarInt,
};
use bytes::Bytes;
use uuid::Uuid;

#[derive(Debug)]
pub struct ReportDetail {
    pub title: String,
    pub description: String,
}

impl McReadBuf for ReportDetail {
    type Output = Self;
    type Error = McStringFieldError;
    fn read_from_buf(data: &mut Bytes) -> Result<ReportDetail, McStringFieldError> {
        let title = McStringField::<128>::read_from_buf(data)?;
        let description = McStringField::<4096>::read_from_buf(data)?;
        Ok(Self { title, description })
    }
}

#[derive(Debug)]
pub struct RegistryEntry {
    pub id: String,
    pub data: Option<McNbtTag>,
}
impl McReadBuf for RegistryEntry {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(buf: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let id = McStringField::<32767>::read_from_buf(buf)?;
        let data = match McBool::read_from_buf(buf)? {
            true => Some(McModernNbtField::read_from_buf(buf)?), // packet used since 764, so it always McModernNbtField
            false => None,
        };
        Ok(Self { id, data })
    }
}
#[derive(Debug)]
pub struct RegistryData {
    pub id: String,
    pub entries: Vec<RegistryEntry>,
}

#[derive(Debug)]
pub enum BuildInLabel {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}
impl From<i32> for BuildInLabel {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::BugReport,
            1 => Self::CommunityGuidelines,
            2 => Self::Support,
            3 => Self::Status,
            4 => Self::Feedback,
            5 => Self::Community,
            6 => Self::Website,
            7 => Self::Forums,
            8 => Self::News,
            9 => Self::Announcements,
            n => panic!("BuildInLabel is only 0-9, but get {}", n),
        }
    }
}
#[derive(Debug)]
pub struct Tag {
    pub tag_name: String,
    pub entries: Vec<McVarInt>,
}
impl McReadBuf for Tag {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(data: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let tag_name = McStringField::<32767>::read_from_buf(data)?;
        let entries = McPrefixedArrayField::<McVarInt>::read_from_buf(data)?;

        Ok(Self { tag_name, entries })
    }
}

#[derive(Debug)]
pub struct RegistryTags {
    pub tag_type: String,
    pub tags: Vec<Tag>,
}

impl McReadBuf for RegistryTags {
    type Output = Self;
    type Error = ParsePacketError;

    fn read_from_buf(data: &mut Bytes) -> Result<Self::Output, Self::Error> {
        let tag_type = McStringField::<32767>::read_from_buf(data)?;
        let tags = McPrefixedArrayField::<Tag>::read_from_buf(data)?;

        Ok(Self { tag_type, tags })
    }
}

#[derive(Debug, Clone)]
pub enum ResourcePackIdent {
    UUID(Uuid),
    Hash(String),
}
impl ResourcePackIdent {
    pub fn from_server_packet(packet: ResourcePackPushPacket) -> Self {
        if let Some(uuid) = packet.uuid {
            Self::UUID(uuid)
        } else {
            Self::Hash(packet.hash)
        }
    }
}
#[derive(Debug)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl McReadBuf for KnownPack {
    type Output = Self;
    type Error = McStringFieldError;

    fn read_from_buf(data: &mut Bytes) -> Result<Self, McStringFieldError> {
        let namespace = McStringField::<32767>::read_from_buf(data)?;
        let id = McStringField::<32767>::read_from_buf(data)?;
        let version = McStringField::<32767>::read_from_buf(data)?;
        Ok(Self {
            namespace,
            id,
            version,
        })
    }
}
