use super::types::DeathLocation;
use super::{ClientBoundPacket, ParsePacketError};
use crate::connection::s2c::try_advance;
use crate::types::{
    Difficulty, GameMode, McBool, McPrefixedArrayField, McReadBuf, McStringField, McVarInt, mc_nbt,
};
use bytes::{Buf, Bytes};

#[derive(Debug, Default)]
pub struct LoginPacket {
    pub entity_id: i32,
    pub game_mode: GameMode,
    pub max_players: i32,
    pub is_hardcore: Option<bool>,
    pub dimension_names: Option<Vec<String>>,
    pub view_distance: Option<u16>,
    pub simulation_distance: Option<u16>,
    pub reduced_debug_info: Option<bool>,
    pub enable_respawn_screen: Option<bool>,
    pub do_limited_crafting: Option<bool>,
    pub enforces_secure_chat: Option<bool>,

    pub dimension_i32: Option<i32>,
    pub dimension_name: Option<String>,

    pub hashed_seed: Option<i64>,
    pub is_debug: Option<bool>,
    pub is_flat: Option<bool>,
    pub difficulty: Option<Difficulty>,
    pub level_type: Option<String>,
    pub death: Option<DeathLocation>,
    pub portal_cooldown: Option<i32>,
    pub sea_level: Option<i32>,
}

impl ClientBoundPacket for LoginPacket {
    const MC_NAME: &str = "login";

    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        let mut packet = LoginPacket::default();
        match protocol {
            ..=46 => {
                packet.entity_id = data.try_get_i32()?;
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                packet.dimension_i32 = Some(data.try_get_i8()? as i32);
                packet.difficulty = Some(Difficulty::try_from(data.try_get_u8()?)?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.level_type = Some(McStringField::<32767>::read_from_buf(&mut data)?);
            }
            47..=107 => {
                packet.entity_id = data.try_get_i32()?;
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                packet.dimension_i32 = Some(data.try_get_i8()? as i32);
                packet.difficulty = Some(Difficulty::try_from(data.try_get_u8()?)?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.level_type = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
            }
            108..=476 => {
                packet.entity_id = data.try_get_i32()?;
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                packet.dimension_i32 = Some(data.try_get_i32()?);
                packet.difficulty = Some(Difficulty::try_from(data.try_get_u8()?)?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.level_type = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
            }
            477..=572 => {
                packet.entity_id = data.try_get_i32()?;
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                packet.dimension_i32 = Some(data.try_get_i32()?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.level_type = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
            }
            573..=733 => {
                packet.entity_id = data.try_get_i32()?;
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                packet.dimension_i32 = Some(data.try_get_i32()?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.level_type = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
            }
            734..=737 => {
                packet.entity_id = data.try_get_i32()?;
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension codec
                _ = McStringField::<32767>::read_from_buf(&mut data)?; // dimension identifier
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);
            }
            738..=750 => {
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension codec
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension nbt
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = data.try_get_u8()? as i32;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);
            }

            751..=756 => {
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension codec
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension nbt
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);
            }
            757..=758 => {
                // + simulation_distance
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension codec (nbt)
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension type (nbt)
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.simulation_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);
            }
            759..=762 => {
                // dimension_type (nbt) changes to (string); + death data (option)
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension codec
                _ = McStringField::<32767>::read_from_buf(&mut data); // dimension type (identifier)
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.simulation_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);

                if McBool::read_from_buf(&mut data)? {
                    packet.death = Some(DeathLocation {
                        dimension_name: McStringField::<32767>::read_from_buf(&mut data)?,
                        location: data.try_get_i64()?,
                    });
                }
            }
            763 => {
                // + portal_cooldown (varint)
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                _ = mc_nbt::read_nbt_from_buf(&mut data, protocol)?; // dimension codec
                _ = McStringField::<32767>::read_from_buf(&mut data); // dimension type (identifier)
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.simulation_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);
                if McBool::read_from_buf(&mut data)? {
                    packet.death = Some(DeathLocation {
                        dimension_name: McStringField::<32767>::read_from_buf(&mut data)?,
                        location: data.try_get_i64()?,
                    });
                }
                packet.portal_cooldown = Some(McVarInt::read_from_buf(&mut data)?.0);
            }
            764..=765 => {
                // Reordering fields + do_limited_crafting (bool)
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.simulation_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.do_limited_crafting = Some(McBool::read_from_buf(&mut data)?);
                _ = McStringField::<32767>::read_from_buf(&mut data); // dimension type (identifier)
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.game_mode = GameMode::try_from(data.try_get_u8()?)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.is_debug = Some(McBool::read_from_buf(&mut data)?);
                packet.is_flat = Some(McBool::read_from_buf(&mut data)?);

                if McBool::read_from_buf(&mut data)? {
                    packet.death = Some(DeathLocation {
                        dimension_name: McStringField::<32767>::read_from_buf(&mut data)?,
                        location: data.try_get_i64()?,
                    });
                }
                packet.portal_cooldown = Some(McVarInt::read_from_buf(&mut data)?.0);
            }
            766..=767 => {
                // dimension_type (identifier) changes to (varint); + enforce_secure_chat (bool)
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.simulation_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.do_limited_crafting = Some(McBool::read_from_buf(&mut data)?);

                _ = McVarInt::read_from_buf(&mut data)?.0; // Dimension type (varint)
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.game_mode = GameMode::try_from(data.try_get_i8()? as u8)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.is_debug = Some(data.try_get_u8()? != 0);
                packet.is_flat = Some(data.try_get_u8()? != 0);

                if McBool::read_from_buf(&mut data)? {
                    packet.death = Some(DeathLocation {
                        dimension_name: McStringField::<32767>::read_from_buf(&mut data)?,
                        location: data.try_get_i64()?,
                    });
                }
                packet.portal_cooldown = Some(McVarInt::read_from_buf(&mut data)?.0);
                packet.enforces_secure_chat = Some(McBool::read_from_buf(&mut data)?);
            }
            768.. => {
                // + sea_level
                packet.entity_id = data.try_get_i32()?;
                packet.is_hardcore = Some(McBool::read_from_buf(&mut data)?);
                packet.dimension_names = Some(
                    McPrefixedArrayField::<McStringField<32767>>::read_from_buf(&mut data)?,
                );
                packet.max_players = McVarInt::read_from_buf(&mut data)?.0;
                packet.view_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.simulation_distance =
                    Some(McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as u16);
                packet.reduced_debug_info = Some(McBool::read_from_buf(&mut data)?);
                packet.enable_respawn_screen = Some(McBool::read_from_buf(&mut data)?);
                packet.do_limited_crafting = Some(McBool::read_from_buf(&mut data)?);

                _ = McVarInt::read_from_buf(&mut data)?.0; // Dimension type (varint)
                packet.dimension_name = Some(McStringField::<32767>::read_from_buf(&mut data)?);
                packet.hashed_seed = Some(data.try_get_i64()?);
                packet.game_mode = GameMode::try_from(data.try_get_i8()? as u8)?;
                try_advance(&mut data, 1)?; // previous_game_mode
                packet.is_debug = Some(data.try_get_u8()? != 0);
                packet.is_flat = Some(data.try_get_u8()? != 0);

                if McBool::read_from_buf(&mut data)? {
                    packet.death = Some(DeathLocation {
                        dimension_name: McStringField::<32767>::read_from_buf(&mut data)?,
                        location: data.try_get_i64()?,
                    });
                }
                packet.portal_cooldown = Some(McVarInt::read_from_buf(&mut data)?.0);
                packet.sea_level = Some(McVarInt::read_from_buf(&mut data)?.0);
                packet.enforces_secure_chat = Some(McBool::read_from_buf(&mut data)?);
            }
        }
        Ok(packet)
    }
}
