use connection::McConnection;
use mc_protocol::connection::s2c::ClientBoundState;
use mc_protocol::states::play::types::Difficulty;
use mc_protocol::types::Player;
use mc_protocol::{connection, is_packet_any, states};
use std::collections::HashSet;
use tokio::net::{TcpStream, ToSocketAddrs};

#[derive(Debug)]
#[allow(dead_code)] // fields used in dbg!()
pub enum ConnectionExitReason {
    DisconnectedByServer(String),
    Encryption,
    Transfer { host: String, port: u16 },
    Successful,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResourcePack {
    pub url: String,
    pub hash: String,
    pub forced: bool,
}

#[allow(dead_code)] // fields used in dbg!()
#[derive(Debug)]
pub struct Mod {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Default)]
pub struct ServerData {
    pub version_name: String,
    pub protocol: i32,
    pub online_players: i32,
    pub max_players: i32,
    pub enforces_secure_chat: Option<bool>,
    pub mods: Option<Vec<Mod>>,
    pub prevents_chat_reports: Option<bool>,
    pub players: HashSet<Player>,
    pub resource_pack: Option<ResourcePack>,
    pub links: Option<Vec<String>>,
    pub code_of_conduct: Option<String>,
    pub commands: Option<Vec<String>>,
    pub difficulty: Option<Difficulty>,
    pub world_data: Option<states::play::s2c::LoginPacket>,
}
#[derive(Debug)]
pub struct ConnectionResult {
    pub exit_reason: ConnectionExitReason,
    pub server_data: ServerData,
}

pub async fn parse_server_data<A: ToSocketAddrs + Clone>(
    addr: A,
    player: &Player,
) -> ConnectionResult {
    let mut server_data = ServerData::default();
    // 1. Handshake with server status response
    let mut conn = McConnection::new(TcpStream::connect(addr.clone()).await.unwrap(), 776);
    handshake_with_status(&mut conn, &mut server_data).await;
    let protocol = server_data.protocol;

    // 2. Trying to connect to a server
    let mut conn = McConnection::new(TcpStream::connect(addr).await.unwrap(), protocol);
    use states::handshake::{C2SHandshakeState, HandshakePacket};
    conn.queue_packet(C2SHandshakeState::Handshake(HandshakePacket {
        server_field: "127.0.0.1".to_string(),
        port_field: 25565,
        protocol: conn.protocol,
        next_state: 2,
    }));

    // 2.1 Login state
    {
        use states::login::c2s::*;
        use states::login::s2c::*;
        conn.queue_packet(C2SLoginState::Hello(HelloPacket::from_player(player)));
        conn.flush().await.unwrap();
        loop {
            let packet: S2CLoginState = conn.read_packet().await.unwrap();
            dbg!(&packet);
            match packet {
                S2CLoginState::LoginDisconnect(p) => {
                    let exit_reason =
                        ConnectionExitReason::DisconnectedByServer(p.reason.formatted());
                    return ConnectionResult {
                        exit_reason,
                        server_data,
                    };
                }
                S2CLoginState::EncryptionRequest(_) => {
                    let exit_reason = ConnectionExitReason::Encryption;
                    return ConnectionResult {
                        exit_reason,
                        server_data,
                    };
                }
                S2CLoginState::LoginFinished(_) => {
                    if protocol >= 764 {
                        conn.send_packet(C2SLoginState::LoginAcknowledged(LoginAcknowledgedPacket))
                            .await
                            .unwrap();
                    }
                    break;
                }
                S2CLoginState::Compression(p) => {
                    conn.enable_compress(p.threshold as usize);
                }
                S2CLoginState::CustomQuery(p) => {
                    conn.send_packet(C2SLoginState::CustomQueryAnswer(
                        CustomQueryAnswerPacket::empty_data(p.message_id),
                    ))
                    .await
                    .unwrap();
                }
                S2CLoginState::CookieRequest(p) => {
                    conn.send_packet(C2SLoginState::CookieResponse(
                        CookieResponsePacket::empty_payload(p.key),
                    ))
                    .await
                    .unwrap();
                }
            };
        }
    }
    // 2.2 Configuration state
    {
        use states::configuration::c2s::*;
        use states::configuration::s2c::*;
        use states::configuration::types::*;

        loop {
            if protocol < 764 {
                break;
            }
            let packet = conn
                .read_filtered_packet(|id| {
                    !is_packet_any!(
                        S2CConfigurationState,
                        id,
                        protocol,
                        [RegistryDataPacket, UpdateTagsPacket]
                    )
                })
                .await
                .unwrap();
            dbg!(&packet);
            match packet {
                S2CConfigurationState::CookieRequest(p) => {
                    conn.send_packet(C2SConfigurationState::CookieResponse(
                        CookieResponsePacket::empty_payload(p.key),
                    ))
                    .await
                    .unwrap();
                }
                S2CConfigurationState::CustomPayload(_) => {}
                S2CConfigurationState::Disconnect(p) => {
                    let exit_reason =
                        ConnectionExitReason::DisconnectedByServer(p.reason.formatted());
                    return ConnectionResult {
                        exit_reason,
                        server_data,
                    };
                }
                S2CConfigurationState::FinishConfiguration(_) => {
                    conn.send_packet(C2SConfigurationState::AckFinishConfiguration(
                        AckFinishConfigurationPacket,
                    ))
                    .await
                    .unwrap();
                    break;
                }
                S2CConfigurationState::KeepAlive(p) => {
                    conn.send_packet(C2SConfigurationState::KeepAliveResponse(
                        KeepAliveResponsePacket {
                            keep_alive_id: p.keep_alive_id,
                        },
                    ))
                    .await
                    .unwrap();
                }
                S2CConfigurationState::Ping(p) => {
                    conn.send_packet(C2SConfigurationState::Pong(PongPacket { id: p.id }))
                        .await
                        .unwrap();
                }
                S2CConfigurationState::ResetChat(_) => {}
                S2CConfigurationState::RegistryData(_) => {}
                S2CConfigurationState::RemoveResourcePack(_) => {}
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
                    conn.flush().await.unwrap();
                }
                S2CConfigurationState::StoreCookie(p) => {
                    conn.send_packet(C2SConfigurationState::CookieResponse(
                        CookieResponsePacket::empty_payload(p.key),
                    ))
                    .await
                    .unwrap();
                }
                S2CConfigurationState::Transfer(p) => {
                    let TransferPacket { host, port } = p;
                    let exit_reason = ConnectionExitReason::Transfer { host, port };
                    return ConnectionResult {
                        exit_reason,
                        server_data,
                    };
                }
                S2CConfigurationState::FeatureFlags(_) => {}
                S2CConfigurationState::UpdateTags(_) => {}
                S2CConfigurationState::KnownPacks(p) => {
                    conn.send_packet(C2SConfigurationState::SelectKnownPacks(
                        SelectKnownPacksPacket {
                            known_packs: p.known_packs,
                        },
                    ))
                    .await
                    .unwrap();
                }
                S2CConfigurationState::CustomReportDetails(_) => {}
                S2CConfigurationState::ServerLinks(p) => {
                    server_data.links = Some(p.links.into_iter().map(|l| l.url).collect());
                }
                S2CConfigurationState::ClearDialog(_) => {}
                S2CConfigurationState::ShowDialog(_) => {}
                S2CConfigurationState::CodeOfConduct(p) => {
                    server_data.code_of_conduct = Some(p.code_of_conduct);
                }
            }
        }
    }
    // 2.3 Play state
    let duration = tokio::time::Duration::from_secs(5);
    let exit_reason =
        tokio::time::timeout(duration, handle_play_state(&mut conn, &mut server_data))
            .await
            .unwrap_or_else(|_| ConnectionExitReason::Successful);

    ConnectionResult {
        exit_reason,
        server_data,
    }
}

pub async fn handshake_with_status(conn: &mut McConnection, server_data: &mut ServerData) {
    use states::handshake::{C2SHandshakeState, HandshakePacket};
    use states::status::{C2SStatusState, S2CStatusState, c2s};
    conn.queue_packet(C2SHandshakeState::Handshake(HandshakePacket {
        server_field: "127.0.0.1".to_string(),
        port_field: 25565,
        protocol: conn.protocol,
        next_state: 1,
    }));
    conn.queue_packet(C2SStatusState::StatusRequest(c2s::StatusRequestPacket));
    conn.flush().await.unwrap();

    let packet: S2CStatusState = conn.read_packet().await.unwrap();
    dbg!(&packet);
    let status_response = match packet {
        S2CStatusState::StatusResponse(packet) => packet,
        S2CStatusState::PongResponse(_) => {
            panic!("Expected status response, but got PongResponse")
        }
    };
    server_data.version_name = status_response.version.name;
    server_data.protocol = status_response.version.protocol;
    server_data.online_players = status_response.players.online;
    server_data.max_players = status_response.players.max;
    server_data
        .players
        .extend(status_response.players.sample.unwrap_or_default());
    server_data.enforces_secure_chat = status_response.enforces_secure_chat;
    server_data.mods = match status_response.forge_data {
        None => None,
        Some(forge_data) => Some(
            forge_data
                .mods
                .into_iter()
                .map(|m| Mod {
                    name: m.mod_id,
                    version: m.mod_marker,
                })
                .collect(),
        ),
    };
    server_data.prevents_chat_reports = status_response.prevents_chat_reports;
}

async fn handle_play_state(
    conn: &mut McConnection,
    server_data: &mut ServerData,
) -> ConnectionExitReason {
    use states::play::c2s::*;
    use states::play::s2c::*;
    use states::play::types::*;
    loop {
        let packet: S2CPlayState = conn.read_packet().await.unwrap();
        dbg!(&packet);
        match packet {
            S2CPlayState::Disconnect(p) => {
                return ConnectionExitReason::DisconnectedByServer(p.reason.formatted());
            }
            S2CPlayState::CookieRequest(p) => {
                conn.send_packet(C2SPlayState::CookieResponse(
                    CookieResponsePacket::empty_payload(p.key),
                ))
                .await
                .unwrap();
            }
            S2CPlayState::KeepAlive(p) => {
                conn.send_packet(C2SPlayState::KeepAliveResponse(KeepAliveResponsePacket {
                    keep_alive_id: p.keep_alive_id,
                }))
                .await
                .unwrap();
            }
            S2CPlayState::Ping(p) => {
                conn.send_packet(C2SPlayState::Pong(PongPacket { id: p.id }))
                    .await
                    .unwrap();
            }
            S2CPlayState::CustomPayload(_) => {}
            S2CPlayState::ChangeDifficulty(p) => server_data.difficulty = Some(p.difficulty),
            S2CPlayState::Commands(p) => {
                server_data.commands = Some(p.command_names);
            }
            S2CPlayState::Login(p) => {
                server_data.world_data = Some(p);
            }
            S2CPlayState::PlayerInfoUpdate(p) => {
                if p.actions.add_player == false {
                    continue;
                }
                let players: HashSet<Player> = p
                    .players
                    .into_iter()
                    .map(|x| {
                        if let Some(uuid) = x.uuid {
                            Player::from_name_and_uuid(x.profile.unwrap().name, uuid).unwrap()
                        } else {
                            Player::from_offline_name(x.profile.unwrap().name)
                        }
                    })
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
                conn.flush().await.unwrap();
            }
            S2CPlayState::StoreCookie(_) => {}
            S2CPlayState::Another(_) => {}
        }
    }
}
