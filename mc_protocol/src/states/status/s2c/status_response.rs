use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{McJsonTextComponent, McReadBuf, McStringField, McVarInt, Player};
use bytes::{Buf, Bytes};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PlayersInfo {
    pub max: i32,
    pub online: i32,
    pub sample: Option<Vec<Player>>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PlayerSample {
    pub(crate) name: String,
    pub(crate) id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ForgeChannel {
    pub res: String,
    pub version: String,
    pub required: bool,
}
#[derive(Deserialize, Debug, Clone)]
pub struct ForgeMod {
    #[serde(alias = "modId", alias = "modid")]
    pub mod_id: String,
    #[serde(alias = "modmarker", alias = "version")]
    pub mod_marker: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ForgeData {
    pub channels: Option<Vec<ForgeChannel>>,

    #[serde(alias = "mods", alias = "modList")]
    pub mods: Vec<ForgeMod>,

    #[serde(rename = "fmlNetworkVersion")]
    pub fml_network_version: Option<i32>,

    #[serde(rename = "d")]
    compressed_data: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StatusResponsePacket {
    pub version: VersionInfo,
    pub players: PlayersInfo,
    pub description: McJsonTextComponent,
    pub favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: Option<bool>,

    #[serde(rename = "forgeData")]
    pub forge_data: Option<ForgeData>,

    #[serde(rename = "preventsChatReports")]
    pub prevents_chat_reports: Option<bool>,

    #[serde(flatten)]
    pub extra_data: HashMap<String, serde_json::Value>,
}
impl ClientBoundPacket for StatusResponsePacket {
    const MC_NAME: &str = "status_response";

    fn parse(mut data: Bytes, _: i32) -> Result<Self, ParsePacketError> {
        let _len_json = McVarInt::read_from_buf(&mut data)?.0;
        if data.len() > 32767 {
            return Err(ParsePacketError::TooLongField {
                max_expected: 32767,
                actual: data.len(),
            });
        }

        let mut status: StatusResponsePacket = serde_json::from_slice(&data)?;
        if let Some(forge_data) = &mut status.forge_data {
            parse_forge_data(forge_data)?;
        }
        Ok(status)
    }
}

fn parse_forge_data(forge_data: &mut ForgeData) -> Result<(), ParsePacketError> {
    let compressed_str = match forge_data.compressed_data.take() {
        Some(data) => data,
        None => return Ok(()),
    };

    let decoded_bytes = decode_optimized_forge_data(&compressed_str)?;
    let mut buf = Bytes::from(decoded_bytes);

    forge_data.fml_network_version = Some(McVarInt::read_from_buf(&mut buf)?.0);
    let _truncated = buf.try_get_u8()? != 0;
    let namespaces_count = McVarInt::read_from_buf(&mut buf)?.0;

    for _ in 0..namespaces_count {
        let packed = McVarInt::read_from_buf(&mut buf)?.0;
        let channels_count = packed >> 1;
        if channels_count < 0 {
            return Err(ParsePacketError::NegativeLength);
        }

        // If true, the mod version string is completely omitted
        let is_mod_missing = (packed & 1) == 1;
        let namespace_name = McStringField::<32767>::read_from_buf(&mut buf)?;

        // Map to old ForgeMod structure
        if !is_mod_missing {
            let mod_version = McStringField::<32767>::read_from_buf(&mut buf)?;
            forge_data.mods.push(ForgeMod {
                mod_id: namespace_name.clone(),
                mod_marker: mod_version,
            });
        }

        let mut channels = Vec::with_capacity(channels_count.min(1024) as usize); // OOM Protection
        for _ in 0..channels_count {
            let channel_name = McStringField::<32767>::read_from_buf(&mut buf)?;
            let channel_version = McVarInt::read_from_buf(&mut buf)?.0;
            let required = buf.try_get_u8()? != 0;
            channels.push(ForgeChannel {
                res: format!("{}:{}", namespace_name, channel_name),
                version: channel_version.to_string(),
                required,
            });
        }
        forge_data.channels = Some(channels);
    }
    Ok(())
}

fn decode_optimized_forge_data(s: &str) -> Result<Vec<u8>, ParsePacketError> {
    let chars: Vec<u16> = s.encode_utf16().collect();

    if chars.len() < 2 {
        return Err(ParsePacketError::UnexpectedEof);
    }

    // First two chars store the target byte array size
    let size0 = chars[0] as usize;
    let size1 = chars[1] as usize;
    let size = size0 | (size1 << 15);

    // 64 KiB maximum preallocate (OOM protection)
    let mut buf = Vec::with_capacity(size.min(64 * 1024));

    let mut string_index = 2;
    let mut buffer: u32 = 0;
    let mut bits_in_buf = 0;

    while string_index < chars.len() {
        // Flush full bytes to the output buffer
        while bits_in_buf >= 8 {
            buf.push((buffer & 0xFF) as u8);
            buffer >>= 8;
            bits_in_buf -= 8;
        }

        // Read next 15 bits from the current character
        let c = chars[string_index] as u32;
        buffer |= (c & 0x7FFF) << bits_in_buf;
        bits_in_buf += 15;
        string_index += 1;
    }

    // Write any leftovers based on the expected total size
    while buf.len() < size {
        buf.push((buffer & 0xFF) as u8);
        buffer >>= 8;
    }

    Ok(buf)
}
