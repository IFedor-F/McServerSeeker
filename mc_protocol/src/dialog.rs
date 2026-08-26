mod cookie_storage;
use cookie_storage::{CookieStorage, CookieStorageError};

pub mod server_dst;

pub use server_dst::ServerDst;
pub mod server_data;

pub use server_data::*;

use crate::connection::s2c::ClientBoundState;
use crate::connection::{McConnection, McConnectionError};
use crate::states::configuration::types::CustomPayloadData;
use crate::types::*;
use crate::{is_packet_any, states};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::timeout;

#[derive(thiserror::Error, Debug)]
pub enum DialogError {
    #[error("McConnection error: {0}")]
    ConnectionError(#[from] McConnectionError),

    #[error("Unsupported protocol version: {0}")]
    UnsupportedProtocolVersion(i32),

    #[error("CookieStorage error: {0}")]
    CookieStorageError(#[from] CookieStorageError),
}
#[derive(Debug)]
enum LoginResult {
    DisconnectByServer { msg: String },
    OnlineMode { should_authenticate: bool },
    Successful,
}
#[derive(Debug)]
enum ConfigurationResult {
    DisconnectByServer { msg: String },
    Transfer { host: String, port: u16 },
    ShowDialog,
    Successful,
}

#[derive(Debug)]
struct PlayResult {
    msg: String,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct ConnectionSettings {
    pub default_protocol: i32,
    pub conn_method: ConnectionMethod,
    pub read_packet_timeout: Duration,
    pub max_config_time: Duration,
    pub max_login_time: Duration,
    pub at_play_time: Duration,
    pub max_transfer_count: u32,
}
impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            default_protocol: 776,
            conn_method: ConnectionMethod::OnlyHandshake,
            read_packet_timeout: Duration::from_secs(3),
            max_login_time: Duration::from_secs(5),
            max_config_time: Duration::from_secs(5),
            at_play_time: Duration::from_secs(3),
            max_transfer_count: 5,
        }
    }
}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum ConnectionMethod {
    OnlyHandshake,
    LoginIfNoMsg,
    JoinIfEmpty,
    Join,
}

pub struct ServerDialog {
    pub dst: ServerDst,
    pub player: Player,
    cookie_storage: CookieStorage,
}

impl ServerDialog {
    pub fn new(dst: ServerDst, player: Player) -> Self {
        Self {
            dst,
            player,
            cookie_storage: CookieStorage::new(),
        }
    }
    fn queue_handshake_packet(&mut self, conn: &mut McConnection, next_state: i32) {
        use states::handshake::*;
        let handshake_packet = HandshakePacket {
            server_field: self.dst.server_name.clone(),
            port_field: self.dst.socket_addr.port(),
            protocol: conn.protocol,
            next_state,
        };
        conn.queue_packet(C2SHandshakeState::Handshake(handshake_packet));
    }

    async fn handshake_with_status_packet(
        &mut self,
        conn: &mut McConnection,
    ) -> Result<ServerData, McConnectionError> {
        use states::status::c2s::*;
        use states::status::s2c::*;
        self.queue_handshake_packet(conn, 1);

        conn.queue_packet(C2SStatusState::StatusRequest(StatusRequestPacket));
        conn.flush().await?;

        // read server status
        let status_resp = conn
            .read_expected_packet::<S2CStatusState, StatusResponsePacket>()
            .await?;

        log::trace!("[{}] {:?}", self.dst.socket_addr, status_resp);
        let mods: Vec<_> = match status_resp.forge_data {
            None => vec![],
            Some(forge_data) => forge_data.mods.into_iter().collect(),
        };

        let players: HashSet<PlayerRecord> = match status_resp.players.sample {
            None => HashSet::new(),
            Some(v) => v.into_iter().map(PlayerRecord::from).collect(),
        };

        let server_data = ServerData {
            description: status_resp.description.formatted(),
            protocol: status_resp.version.protocol,
            version_name: status_resp.version.name,
            online_players: status_resp.players.online,
            max_players: status_resp.players.max,
            players,
            enforces_secure_chat: status_resp.enforces_secure_chat.unwrap_or(false),
            no_chat_reports: status_resp.prevents_chat_reports.unwrap_or(false),
            mods,
            brand: None,
            links: vec![],
            code_of_conduct: None,
            features: vec![],
            resource_pack: None,
            channels: HashSet::new(),
            registered_channels: HashSet::new(),
            difficulty: None,
            dimension: None,
            view_distance: None,
            simulation_distance: None,
            is_hardcore: None,
            reduced_debug_info: None,
            do_limited_crafting: None,
            commands: vec![],
            hashed_seed: None,
            gamemode: None,
            is_flat: None,
        };
        Ok(server_data)
    }

    async fn handle_login_state(
        &mut self,
        conn: &mut McConnection,
    ) -> Result<LoginResult, DialogError> {
        use states::login::c2s::*;
        use states::login::s2c::*;
        conn.send_packet(C2SLoginState::Hello(HelloPacket::from_player(&self.player)))
            .await?;

        loop {
            let packet: S2CLoginState = conn.read_packet().await?;
            log::trace!("[{}] {:?}", self.dst.socket_addr, packet);
            match packet {
                S2CLoginState::LoginDisconnect(p) => {
                    return Ok(LoginResult::DisconnectByServer {
                        msg: p.reason.formatted(),
                    });
                }
                S2CLoginState::EncryptionRequest(p) => {
                    return Ok(LoginResult::OnlineMode {
                        should_authenticate: p.should_authenticate.unwrap_or(true),
                    });
                }
                S2CLoginState::LoginFinished(_) => {
                    if conn.protocol >= 764 {
                        conn.send_packet(C2SLoginState::LoginAcknowledged(LoginAcknowledgedPacket))
                            .await?;
                    }
                    break;
                }
                S2CLoginState::Compression(p) => {
                    conn.enable_compress(p.threshold as usize);
                }
                S2CLoginState::CustomQuery(p) => {
                    let ans = CustomQueryAnswerPacket::empty_data(p.message_id);
                    conn.send_packet(C2SLoginState::CustomQueryAnswer(ans))
                        .await?;
                }
                S2CLoginState::CookieRequest(p) => {
                    let ans = self.cookie_storage.format_packet(p.key);
                    conn.send_packet(C2SLoginState::CookieResponse(ans)).await?;
                }
            }
        }
        Ok(LoginResult::Successful)
    }

    async fn handle_configuration_state(
        &mut self,
        conn: &mut McConnection,
        server_data: &mut ServerData,
        send_finish: bool,
    ) -> Result<ConfigurationResult, DialogError> {
        use states::configuration::c2s::*;
        use states::configuration::s2c::*;
        use states::configuration::types::*;
        loop {
            let protocol = conn.protocol;
            let packet: S2CConfigurationState = conn
                .skip_until_filter(|id| {
                    !is_packet_any!(
                        S2CConfigurationState,
                        id,
                        protocol,
                        [
                            ResetChatPacket,
                            RegistryDataPacket,
                            RemoveResourcePackPacket,
                            UpdateTagsPacket,
                            CustomReportDetailsPacket,
                        ]
                    )
                })
                .await?;

            log::trace!("[{}] {:?}", self.dst.socket_addr, packet);
            match packet {
                S2CConfigurationState::ResetChat(_) => unreachable!(),
                S2CConfigurationState::RegistryData(_) => unreachable!(),
                S2CConfigurationState::RemoveResourcePack(_) => unreachable!(),
                S2CConfigurationState::UpdateTags(_) => unreachable!(),
                S2CConfigurationState::CustomReportDetails(_) => unreachable!(),

                S2CConfigurationState::StoreCookie(p) => {
                    self.cookie_storage.try_put(p.key, p.payload)?
                }

                S2CConfigurationState::CookieRequest(p) => {
                    let ans = self.cookie_storage.format_packet(p.key);
                    conn.send_packet(C2SConfigurationState::CookieResponse(ans))
                        .await?;
                }
                S2CConfigurationState::CustomPayload(p) => match p.data {
                    CustomPayloadData::MinecraftBrand(brand) => {
                        server_data.brand = Some(brand);
                    }
                    CustomPayloadData::MinecraftRegister { channels } => {
                        server_data.registered_channels.extend(channels);
                    }
                    CustomPayloadData::Unrecognized(_) => {
                        server_data.channels.insert(p.channel);
                    }
                },
                S2CConfigurationState::Disconnect(p) => {
                    return Ok(ConfigurationResult::DisconnectByServer {
                        msg: p.reason.formatted(),
                    });
                }
                S2CConfigurationState::KeepAlive(p) => {
                    let ans = KeepAliveResponsePacket {
                        keep_alive_id: p.keep_alive_id,
                    };
                    conn.send_packet(C2SConfigurationState::KeepAliveResponse(ans))
                        .await?;
                }
                S2CConfigurationState::Ping(p) => {
                    let ans = PongPacket { id: p.id };
                    conn.send_packet(C2SConfigurationState::Pong(ans)).await?;
                }
                S2CConfigurationState::AddResourcePack(p) => {
                    server_data.resource_pack = Some(ResourcePack {
                        url: p.url.clone(),
                        hash: p.hash.clone(),
                        forced: p.forced.unwrap_or(true),
                    });
                    let ident = ResourcePackIdent::from_server_packet(p);
                    let packet_accepted = C2SConfigurationState::ResourcePackResponse(
                        ResourcePackResponsePacket::accepted(ident.clone()),
                    );
                    let packet_downloaded = C2SConfigurationState::ResourcePackResponse(
                        ResourcePackResponsePacket::successfully_downloaded(ident),
                    );
                    conn.queue_packet(packet_accepted);
                    conn.queue_packet(packet_downloaded);
                    conn.flush().await?;
                }
                S2CConfigurationState::Transfer(p) => {
                    let TransferPacket { host, port } = p;
                    return Ok(ConfigurationResult::Transfer { host, port });
                }
                S2CConfigurationState::FeatureFlags(p) => {
                    server_data.features = p.features;
                }
                S2CConfigurationState::KnownPacks(_) => {
                    let ans = SelectKnownPacksPacket::empty();
                    conn.send_packet(C2SConfigurationState::SelectKnownPacks(ans))
                        .await?;
                }
                S2CConfigurationState::ServerLinks(p) => {
                    server_data.links.extend(p.links.into_iter().map(|l| l.url))
                }
                S2CConfigurationState::ClearDialog(_) => {}
                S2CConfigurationState::ShowDialog(_) => return Ok(ConfigurationResult::ShowDialog),
                S2CConfigurationState::CodeOfConduct(p) => {
                    server_data.code_of_conduct = Some(p.code_of_conduct);
                    let ans = AcceptCodeOfConductPacket;
                    conn.send_packet(C2SConfigurationState::AcceptCodeOfConduct(ans))
                        .await?;
                }
                S2CConfigurationState::FinishConfiguration(_) => {
                    if send_finish {
                        let ans = AckFinishConfigurationPacket;
                        conn.send_packet(C2SConfigurationState::AckFinishConfiguration(ans))
                            .await?;
                    }
                    break;
                }
            }
        }
        Ok(ConfigurationResult::Successful)
    }
    async fn handle_play_state(
        &mut self,
        conn: &mut McConnection,
        server_data: &mut ServerData,
    ) -> Result<PlayResult, DialogError> {
        use states::play::c2s::*;
        use states::play::s2c::*;
        use states::play::types::*;

        loop {
            let protocol = conn.protocol;
            let packet: S2CPlayState = conn
                .skip_until_filter(|id| {
                    !is_packet_any!(S2CPlayState, id, protocol, [AnotherPacket])
                })
                .await?;

            log::trace!("[{}] {:?}", self.dst.socket_addr, packet);
            match packet {
                S2CPlayState::Another(_) => unreachable!(),

                S2CPlayState::Disconnect(p) => {
                    return Ok(PlayResult {
                        msg: p.reason.formatted(),
                    });
                }
                S2CPlayState::KeepAlive(p) => {
                    let ans = KeepAliveResponsePacket {
                        keep_alive_id: p.keep_alive_id,
                    };
                    conn.send_packet(C2SPlayState::KeepAliveResponse(ans))
                        .await?;
                }
                S2CPlayState::Ping(p) => {
                    let ans = PongPacket { id: p.id };
                    conn.send_packet(C2SPlayState::Pong(ans)).await?;
                }
                S2CPlayState::CustomPayload(p) => match p.data {
                    CustomPayloadData::MinecraftBrand(brand) => {
                        server_data.brand = Some(brand);
                    }
                    CustomPayloadData::MinecraftRegister { channels } => {
                        server_data.registered_channels.extend(channels);
                    }
                    CustomPayloadData::Unrecognized(_) => {
                        server_data.channels.insert(p.channel);
                    }
                },
                S2CPlayState::ChangeDifficulty(p) => server_data.difficulty = Some(p.difficulty),
                S2CPlayState::Commands(p) => {
                    server_data.commands = p.command_names;
                }
                S2CPlayState::CookieRequest(p) => {
                    let ans = self.cookie_storage.format_packet(p.key);
                    conn.send_packet(C2SPlayState::CookieResponse(ans)).await?;
                }
                S2CPlayState::Login(p) => {
                    server_data.max_players = p.max_players;
                    server_data.enforces_secure_chat = p.enforces_secure_chat.unwrap_or(false);

                    server_data.hashed_seed = p.hashed_seed;
                    server_data.gamemode = Some(p.game_mode);
                    if server_data.difficulty.is_none() {
                        server_data.difficulty = p.difficulty
                    }
                    server_data.dimension = match p.dimension_name {
                        None => {
                            let dim_n = p.dimension_i32.unwrap(); // value is expected, because dimension if always in login packet, but can be in i32 or string
                            match dim_n {
                                -1 => Some("minecraft:the_nether".to_string()),
                                0 => Some("minecraft:overworld".to_string()),
                                1 => Some("minecraft:the_end".to_string()),
                                _ => None, // if dimension is custom - return None
                            }
                        }
                        Some(name) => Some(name),
                    };
                    server_data.view_distance = p.view_distance;
                    server_data.simulation_distance = p.simulation_distance;
                    server_data.is_hardcore = p.is_hardcore;
                    server_data.reduced_debug_info = p.reduced_debug_info;
                    server_data.do_limited_crafting = p.do_limited_crafting;
                    server_data.is_flat = match p.is_flat {
                        Some(_) => p.is_flat,
                        None => p.level_type.and_then(|t| Some(t == "flat")),
                    }
                }
                S2CPlayState::PlayerInfoUpdate(p) => {
                    if !p.actions.add_player {
                        continue;
                    }
                    let players: HashSet<PlayerRecord> = p
                        .players_info
                        .into_iter()
                        .map(|x| {
                            let profile = x.profile.unwrap(); // expected because action has `add_player`
                            if let Some(uuid) = x.uuid {
                                let player = Player::from_name_and_uuid(profile.name, uuid)?;
                                Ok::<PlayerRecord, PlayerParseError>(PlayerRecord {
                                    player,
                                    game_mode: x.game_mode,
                                    ping: x.ping,
                                })
                            } else {
                                let player = Player::from_offline_name(profile.name)?;
                                Ok(PlayerRecord {
                                    player,
                                    game_mode: x.game_mode,
                                    ping: x.ping,
                                })
                            }
                        })
                        .filter_map(Result::ok)
                        .filter(|r| r.player.name != self.player.name)
                        .collect();
                    server_data.players.extend(players)
                }
                S2CPlayState::ResourcePackPush(p) => {
                    server_data.resource_pack = Some(ResourcePack {
                        url: p.url.clone(),
                        hash: p.hash.clone(),
                        forced: p.forced.unwrap_or(true),
                    });

                    let ident = ResourcePackIdent::from_server_packet(p);
                    let packet_accepted = C2SPlayState::ResourcePackResponse(
                        ResourcePackResponsePacket::accepted(ident.clone()),
                    );
                    let packet_downloaded = C2SPlayState::ResourcePackResponse(
                        ResourcePackResponsePacket::successfully_downloaded(ident),
                    );
                    conn.queue_packet(packet_accepted);
                    conn.queue_packet(packet_downloaded);
                    conn.flush().await?;
                }
                S2CPlayState::StoreCookie(p) => {
                    self.cookie_storage.try_put(p.key, p.payload)?;
                }
                S2CPlayState::ServerLinks(p) => {
                    server_data.links.extend(p.links.into_iter().map(|l| l.url))
                }
            }
        }
    }

    pub async fn connect(mut self, settings: ConnectionSettings) -> ConnectionResult {
        let ConnectionSettings {
            default_protocol,
            conn_method,
            read_packet_timeout,
            max_config_time,
            max_login_time,
            at_play_time,
            max_transfer_count,
        } = settings;

        log::debug!(
            "[{}] connecting via name '{}'",
            self.dst.socket_addr,
            self.dst.server_name
        );

        // 1. Try Handshake
        let mut conn = match self
            .dst
            .make_conn(default_protocol, read_packet_timeout)
            .await
        {
            Ok(conn) => conn,
            Err(e) => {
                log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);
                return ConnectionResult::NoHandshake(DialogError::ConnectionError(e));
            }
        };
        log::debug!("[{}] tcp connected", self.dst.socket_addr);

        let status_result = self.handshake_with_status_packet(&mut conn).await;
        let mut data = match status_result {
            Err(e) => {
                return ConnectionResult::NoHandshake(DialogError::ConnectionError(e));
            }
            Ok(data) => data,
        };
        let protocol = data.protocol;
        log::debug!("[{}] server protocol is {}", self.dst.socket_addr, protocol);

        // If only handshake
        if conn_method == ConnectionMethod::OnlyHandshake
            || (conn_method == ConnectionMethod::LoginIfNoMsg && protocol < 764)
            || (conn_method == ConnectionMethod::JoinIfEmpty
                && protocol < 764
                && data.online_players > 0)
        {
            return ConnectionResult::Successful { data };
        }

        if protocol < 5 || protocol > 776 {
            return ConnectionResult::DisconnectAtLogin {
                data,
                reason: DisconnectLoginReason::DialogError(
                    DialogError::UnsupportedProtocolVersion(protocol),
                ),
            };
        }

        // 2. Try joining server
        conn = match self.dst.make_conn(protocol, read_packet_timeout).await {
            Ok(conn) => conn,
            Err(e) => {
                log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);
                return ConnectionResult::DisconnectAtLogin {
                    data,
                    reason: DisconnectLoginReason::DialogError(DialogError::ConnectionError(e)),
                };
            }
        };
        log::debug!("[{}] switching to login state", self.dst.socket_addr);
        self.queue_handshake_packet(&mut conn, 2);

        // Loop for transfer packet logic
        let mut transfer_count = 0;
        loop {
            // login state
            let login_result = timeout(max_login_time, self.handle_login_state(&mut conn)).await;

            match login_result {
                Ok(Err(e)) => {
                    log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);
                    return ConnectionResult::DisconnectAtLogin {
                        data,
                        reason: DisconnectLoginReason::DialogError(e),
                    };
                }
                Ok(Ok(LoginResult::Successful)) => {}
                Ok(Ok(LoginResult::DisconnectByServer { msg })) => {
                    log::debug!("[{}] disconnected by server: {}", self.dst.socket_addr, msg);
                    return ConnectionResult::DisconnectAtLogin {
                        data,
                        reason: DisconnectLoginReason::DisconnectByServer { msg },
                    };
                }
                Ok(Ok(LoginResult::OnlineMode {
                    should_authenticate,
                })) => {
                    log::debug!(
                        "[{}] leave server because auth required",
                        self.dst.socket_addr
                    );
                    return ConnectionResult::DisconnectAtLogin {
                        data,
                        reason: DisconnectLoginReason::OnlineMode {
                            should_authenticate,
                        },
                    };
                }
                Err(_) => {
                    return ConnectionResult::DisconnectAtLogin {
                        data,
                        reason: DisconnectLoginReason::Timeout,
                    };
                }
            }

            // Check should we switch to configuration state
            if protocol < 764 {
                break;
            }

            log::debug!(
                "[{}] switching to configuration state",
                self.dst.socket_addr
            );

            let send_finish = match conn_method {
                ConnectionMethod::JoinIfEmpty => data.online_players == 0,
                ConnectionMethod::Join => true,
                _ => false,
            };

            // configuration state
            let configuration_result = timeout(
                max_config_time,
                self.handle_configuration_state(&mut conn, &mut data, send_finish),
            )
            .await;
            match configuration_result {
                Ok(Err(e)) => {
                    log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);

                    return ConnectionResult::DisconnectAtConfiguration {
                        data,
                        reason: DisconnectConfigurationReason::DialogError(e),
                    };
                }
                Ok(Ok(ConfigurationResult::DisconnectByServer { msg })) => {
                    log::debug!("[{}] disconnected by server: {}", self.dst.socket_addr, msg);
                    return ConnectionResult::DisconnectAtConfiguration {
                        data,
                        reason: DisconnectConfigurationReason::DisconnectByServer { msg },
                    };
                }
                Ok(Ok(ConfigurationResult::ShowDialog)) => {
                    log::debug!(
                        "[{}] leave server because dialog was sent",
                        self.dst.socket_addr
                    );

                    return ConnectionResult::DisconnectAtConfiguration {
                        data,
                        reason: DisconnectConfigurationReason::ShowDialog,
                    };
                }
                Ok(Ok(ConfigurationResult::Transfer { host, port })) => {
                    log::debug!(
                        "[{}] transfer required to {}:{}",
                        self.dst.socket_addr,
                        host,
                        port
                    );
                    if transfer_count > max_transfer_count {
                        return ConnectionResult::DisconnectAtConfiguration {
                            data,
                            reason: DisconnectConfigurationReason::TooMuchTransfers {
                                count: transfer_count,
                            },
                        };
                    }
                    transfer_count += 1;
                    self.dst = match ServerDst::from_host_and_port(host, port).await {
                        Ok(dst) => dst,
                        Err(e) => {
                            log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);
                            return ConnectionResult::DisconnectAtConfiguration {
                                data,
                                reason: DisconnectConfigurationReason::CantResolveHost(e),
                            };
                        }
                    };
                    conn = match self.dst.make_conn(protocol, read_packet_timeout).await {
                        Ok(conn) => conn,
                        Err(e) => {
                            log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);
                            return ConnectionResult::DisconnectAtConfiguration {
                                data,
                                reason: DisconnectConfigurationReason::DialogError(
                                    DialogError::ConnectionError(e),
                                ),
                            };
                        }
                    };
                    // back to login state
                    self.queue_handshake_packet(&mut conn, 3);
                    continue;
                }

                Ok(Ok(ConfigurationResult::Successful)) => break,
                Err(_) => {
                    return ConnectionResult::DisconnectAtConfiguration {
                        data,
                        reason: DisconnectConfigurationReason::Timeout,
                    };
                }
            }
        }

        if conn_method < ConnectionMethod::JoinIfEmpty
            || (conn_method == ConnectionMethod::JoinIfEmpty && data.online_players > 0)
        {
            log::debug!(
                "[{}] disconnect from server because there is some players",
                self.dst.socket_addr
            );
            return ConnectionResult::Successful { data };
        }

        log::debug!("[{}] switching to play state", self.dst.socket_addr);
        let play_result =
            tokio::time::timeout(at_play_time, self.handle_play_state(&mut conn, &mut data)).await;
        match play_result {
            Ok(Err(e)) => {
                log::debug!("[{}] dialog error: {}", self.dst.socket_addr, e);

                return ConnectionResult::DisconnectAtPlay {
                    data,
                    reason: DisconnectPlayReason::DialogError(e),
                };
            }
            Ok(Ok(PlayResult { msg })) => {
                log::debug!("[{}] disconnected by server: {}", self.dst.socket_addr, msg);
                return ConnectionResult::DisconnectAtPlay {
                    reason: DisconnectPlayReason::DisconnectByServer { msg },
                    data,
                };
            }
            _ => {}
        }
        ConnectionResult::Successful { data }
    }
}
