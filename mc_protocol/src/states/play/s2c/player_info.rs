use super::types::{ChatSession, McChatComponent, PlayerActions, PlayerCrypto, PlayerInfo};
use super::{ClientBoundPacket, ParsePacketError};
use crate::types::{GameProfile, McBool, McReadBuf, McStringField, McTextComponent, McVarInt};
use bytes::{Buf, Bytes};
use uuid::Uuid;

#[derive(Debug)]
pub struct PlayerInfoUpdatePacket {
    pub actions: PlayerActions,
    pub players: Vec<PlayerInfo>,
}

impl ClientBoundPacket for PlayerInfoUpdatePacket {
    const MC_NAME: &str = "player_info_update";

    fn parse(mut data: Bytes, protocol: i32) -> Result<Self, ParsePacketError> {
        match protocol {
            ..=46 => {
                let name = McStringField::<16>::read_from_buf(&mut data)?;
                let online = McBool::read_from_buf(&mut data)?;
                let ping = data.try_get_i16()?;

                let mut actions = PlayerActions::default();
                if online {
                    actions.add_player = true;
                    actions.update_latency = true;
                } else {
                    actions.remove_player = true;
                }

                let mut player = PlayerInfo {
                    uuid: None,
                    ping: Some(ping as i32),
                    ..PlayerInfo::default()
                };
                player.profile = Some(GameProfile {
                    name,
                    properties: vec![],
                });

                Ok(Self {
                    actions,
                    players: vec![player],
                })
            }

            47..=758 => {
                // now it's array of action + action type is enum
                let action_raw = McVarInt::read_from_buf(&mut data)?.0;
                let mut actions = PlayerActions::default();

                match action_raw {
                    0 => {
                        actions.add_player = true;
                        actions.update_game_mode = true;
                        actions.update_latency = true;
                        actions.update_display_name = true;
                    }
                    1 => actions.update_game_mode = true,
                    2 => actions.update_latency = true,
                    3 => actions.update_display_name = true,
                    4 => actions.remove_player = true,
                    _ => {
                        return Err(ParsePacketError::TooLongField {
                            max_expected: 4,
                            actual: action_raw as usize,
                        });
                    }
                }

                let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let mut players = Vec::with_capacity(count.min(1024)); // OOM protection

                for _ in 0..count {
                    let uuid = Some(Uuid::from_u128(data.try_get_u128()?));
                    let mut player = PlayerInfo {
                        uuid,
                        ..Default::default()
                    };

                    match action_raw {
                        0 => {
                            player.profile = Some(GameProfile::read_from_buf(&mut data)?);
                            player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0);
                            player.ping = Some(McVarInt::read_from_buf(&mut data)?.0);
                            if McBool::read_from_buf(&mut data)? {
                                player.display_name = Some(McChatComponent::String(
                                    McStringField::<32767>::read_from_buf(&mut data)?,
                                ));
                            }
                        }
                        1 => player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0),
                        2 => player.ping = Some(McVarInt::read_from_buf(&mut data)?.0),
                        3 => {
                            if McBool::read_from_buf(&mut data)? {
                                player.display_name = Some(McChatComponent::String(
                                    McStringField::<32767>::read_from_buf(&mut data)?,
                                ));
                            }
                        }
                        4 => {}
                        _ => unreachable!(),
                    }
                    players.push(player);
                }
                Ok(Self { actions, players })
            }

            759..=760 => {
                // added crypto field
                let action_raw = McVarInt::read_from_buf(&mut data)?.0;
                let mut actions = PlayerActions::default();

                match action_raw {
                    0 => {
                        actions.add_player = true;
                        actions.update_game_mode = true;
                        actions.update_latency = true;
                        actions.update_display_name = true;
                    }
                    1 => actions.update_game_mode = true,
                    2 => actions.update_latency = true,
                    3 => actions.update_display_name = true,
                    4 => actions.remove_player = true,
                    _ => {
                        return Err(ParsePacketError::TooLongField {
                            max_expected: 4,
                            actual: action_raw as usize,
                        });
                    }
                }

                let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let mut players = Vec::with_capacity(count.min(1024));

                for _ in 0..count {
                    let uuid = Some(Uuid::from_u128(data.try_get_u128()?));
                    let mut player = PlayerInfo {
                        uuid,
                        ..Default::default()
                    };

                    match action_raw {
                        0 => {
                            player.profile = Some(GameProfile::read_from_buf(&mut data)?);
                            player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0);
                            player.ping = Some(McVarInt::read_from_buf(&mut data)?.0);
                            if McBool::read_from_buf(&mut data)? {
                                player.display_name = Some(McChatComponent::String(
                                    McStringField::<32767>::read_from_buf(&mut data)?,
                                ));
                            }
                            if McBool::read_from_buf(&mut data)? {
                                player.crypto = Some(PlayerCrypto::read_from_buf(&mut data)?);
                            }
                        }
                        1 => player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0),
                        2 => player.ping = Some(McVarInt::read_from_buf(&mut data)?.0),
                        3 => {
                            if McBool::read_from_buf(&mut data)? {
                                player.display_name = Some(McChatComponent::String(
                                    McStringField::<32767>::read_from_buf(&mut data)?,
                                ));
                            }
                        }
                        4 => {}
                        _ => unreachable!(),
                    }
                    players.push(player);
                }
                Ok(Self { actions, players })
            }

            761..=764 => {
                // multiple updates with bitmask (action type now is bismask), `remove_player` action is removed
                let action_flags = data.try_get_u8()?;
                let actions = PlayerActions {
                    add_player: (action_flags & 0x01) != 0,
                    initialize_chat: (action_flags & 0x02) != 0,
                    update_game_mode: (action_flags & 0x04) != 0,
                    update_listed: (action_flags & 0x08) != 0,
                    update_latency: (action_flags & 0x10) != 0,
                    update_display_name: (action_flags & 0x20) != 0,
                    ..Default::default()
                };

                let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let mut players = Vec::with_capacity(count.min(1024));

                for _ in 0..count {
                    let uuid = Some(Uuid::from_u128(data.try_get_u128()?));
                    let mut player = PlayerInfo {
                        uuid,
                        ..Default::default()
                    };

                    if actions.add_player {
                        player.profile = Some(GameProfile::read_from_buf(&mut data)?);
                    }
                    if actions.initialize_chat {
                        if McBool::read_from_buf(&mut data)? {
                            player.chat_session = Some(ChatSession::read_from_buf(&mut data)?);
                        }
                    }
                    if actions.update_game_mode {
                        player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_listed {
                        player.listed = Some(McBool::read_from_buf(&mut data)?);
                    }
                    if actions.update_latency {
                        player.ping = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_display_name {
                        if McBool::read_from_buf(&mut data)? {
                            player.display_name =
                                Some(McChatComponent::String(
                                    McStringField::<32767>::read_from_buf(&mut data)?,
                                ));
                        }
                    }
                    players.push(player);
                }
                Ok(Self { actions, players })
            }

            765..=767 => {
                // display_name now is nbt (Text component)
                let action_flags = data.try_get_u8()?;
                let actions = PlayerActions {
                    add_player: (action_flags & 0x01) != 0,
                    initialize_chat: (action_flags & 0x02) != 0,
                    update_game_mode: (action_flags & 0x04) != 0,
                    update_listed: (action_flags & 0x08) != 0,
                    update_latency: (action_flags & 0x10) != 0,
                    update_display_name: (action_flags & 0x20) != 0,
                    ..Default::default()
                };

                let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let mut players = Vec::with_capacity(count.min(1024));

                for _ in 0..count {
                    let uuid = Some(Uuid::from_u128(data.try_get_u128()?));
                    let mut player = PlayerInfo {
                        uuid,
                        ..Default::default()
                    };

                    if actions.add_player {
                        player.profile = Some(GameProfile::read_from_buf(&mut data)?);
                    }
                    if actions.initialize_chat {
                        if McBool::read_from_buf(&mut data)? {
                            player.chat_session = Some(ChatSession::read_from_buf(&mut data)?);
                        }
                    }
                    if actions.update_game_mode {
                        player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_listed {
                        player.listed = Some(McBool::read_from_buf(&mut data)?);
                    }
                    if actions.update_latency {
                        player.ping = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_display_name {
                        if McBool::read_from_buf(&mut data)? {
                            player.display_name = Some(McChatComponent::Nbt(
                                McTextComponent::read_from_buf_with_protocol(&mut data, protocol)?,
                            ));
                        }
                    }
                    players.push(player);
                }
                Ok(Self { actions, players })
            }

            768 => {
                let action_flags = data.try_get_u8()?;
                let actions = PlayerActions {
                    add_player: (action_flags & 0x01) != 0,
                    initialize_chat: (action_flags & 0x02) != 0,
                    update_game_mode: (action_flags & 0x04) != 0,
                    update_listed: (action_flags & 0x08) != 0,
                    update_latency: (action_flags & 0x10) != 0,
                    update_display_name: (action_flags & 0x20) != 0,
                    update_list_priority: (action_flags & 0x40) != 0,
                    ..Default::default()
                };

                let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let mut players = Vec::with_capacity(count.min(1024));

                for _ in 0..count {
                    let uuid = Some(Uuid::from_u128(data.try_get_u128()?));
                    let mut player = PlayerInfo {
                        uuid,
                        ..Default::default()
                    };

                    if actions.add_player {
                        player.profile = Some(GameProfile::read_from_buf(&mut data)?);
                    }
                    if actions.initialize_chat {
                        if McBool::read_from_buf(&mut data)? {
                            player.chat_session = Some(ChatSession::read_from_buf(&mut data)?);
                        }
                    }
                    if actions.update_game_mode {
                        player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_listed {
                        player.listed = Some(McBool::read_from_buf(&mut data)?);
                    }
                    if actions.update_latency {
                        player.ping = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_display_name {
                        if McBool::read_from_buf(&mut data)? {
                            player.display_name = Some(McChatComponent::Nbt(
                                McTextComponent::read_from_buf_with_protocol(&mut data, protocol)?,
                            ));
                        }
                    }
                    if actions.update_list_priority {
                        player.list_priority = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }

                    players.push(player);
                }
                Ok(Self { actions, players })
            }

            769.. => {
                let action_flags = data.try_get_u8()?;
                let actions = PlayerActions {
                    add_player: (action_flags & 0x01) != 0,
                    initialize_chat: (action_flags & 0x02) != 0,
                    update_game_mode: (action_flags & 0x04) != 0,
                    update_listed: (action_flags & 0x08) != 0,
                    update_latency: (action_flags & 0x10) != 0,
                    update_display_name: (action_flags & 0x20) != 0,
                    update_list_priority: (action_flags & 0x40) != 0,
                    update_hat: (action_flags & 0x80) != 0,
                    ..Default::default()
                };

                let count = McVarInt::read_from_buf(&mut data)?.with_min_check(0)?.0 as usize;
                let mut players = Vec::with_capacity(count.min(1024));

                for _ in 0..count {
                    let uuid = Some(Uuid::from_u128(data.try_get_u128()?));
                    let mut player = PlayerInfo {
                        uuid,
                        ..Default::default()
                    };

                    if actions.add_player {
                        player.profile = Some(GameProfile::read_from_buf(&mut data)?);
                    }
                    if actions.initialize_chat {
                        if McBool::read_from_buf(&mut data)? {
                            player.chat_session = Some(ChatSession::read_from_buf(&mut data)?);
                        }
                    }
                    if actions.update_game_mode {
                        player.game_mode = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_listed {
                        player.listed = Some(McBool::read_from_buf(&mut data)?);
                    }
                    if actions.update_latency {
                        player.ping = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_display_name {
                        if McBool::read_from_buf(&mut data)? {
                            player.display_name = Some(McChatComponent::Nbt(
                                McTextComponent::read_from_buf_with_protocol(&mut data, protocol)?,
                            ));
                        }
                    }
                    if actions.update_list_priority {
                        player.list_priority = Some(McVarInt::read_from_buf(&mut data)?.0);
                    }
                    if actions.update_hat {
                        player.show_hat = Some(McBool::read_from_buf(&mut data)?);
                    }

                    players.push(player);
                }
                Ok(Self { actions, players })
            }
        }
    }
}
