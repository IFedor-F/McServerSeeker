use data_core::proto::scanner as pb;
use mc_protocol::dialog::{
    ConnectionMethod, ConnectionResult, DisconnectConfigurationReason, DisconnectLoginReason,
    DisconnectPlayReason, ServerData,
};
use mc_protocol::types::Player;
use mc_protocol::types::player::OnlineType;

pub struct ExtraServerData {
    pub is_whitelist: Option<bool>,
    pub is_online_mode: Option<bool>,
    pub offline_auth: Option<bool>,
    pub connection_result: pb::ConnectionResult,
    pub player_name: String,
}

impl ExtraServerData {
    pub fn parse(
        player: Player,
        connection_method: ConnectionMethod,
        connection_result: &ConnectionResult,
    ) -> Self {
        let player_name = player.name.clone();
        let mut is_whitelist: Option<bool> = None;
        let mut is_online_mode: Option<bool> = None;
        let mut offline_auth: Option<bool> = None;

        match connection_result {
            ConnectionResult::NoHandshake(_) => {
                panic!("Expect result with some data, but get NoHandshake")
            }
            ConnectionResult::DisconnectAtLogin { data, reason } => {
                match reason {
                    DisconnectLoginReason::OnlineMode {
                        should_authenticate,
                    } => {
                        is_online_mode = Some(should_authenticate.clone());
                    }
                    DisconnectLoginReason::DisconnectByServer { msg } => {
                        if msg.contains("whitelist") {
                            is_whitelist = Some(true);
                            is_online_mode = Some(false) // because online check if before whitelist
                        };
                    }
                    DisconnectLoginReason::DialogError(e) => {
                        log::warn!("dialog error: {:?}", e);
                        is_online_mode = check_online_mode_by_player_list(data);
                    }
                    DisconnectLoginReason::Timeout => {
                        log::debug!("disconnect because timeout for state login");
                        is_online_mode = check_online_mode_by_player_list(data);
                    }
                }
                Self {
                    is_whitelist,
                    is_online_mode,
                    offline_auth,
                    connection_result: pb::ConnectionResult::LoginDisconnect,
                    player_name,
                }
            }

            ConnectionResult::DisconnectAtConfiguration { data: _, reason } => {
                match reason {
                    DisconnectConfigurationReason::DisconnectByServer { msg } => {
                        log::debug!("unexpected server disconnect in configuration: {}", msg)
                    }
                    DisconnectConfigurationReason::ShowDialog => {
                        log::debug!("disconnect because dialog was sent")
                    }
                    DisconnectConfigurationReason::TooMuchTransfers { count } => {
                        log::debug!(
                            "disconnect from server due too much transfers attempts: {}",
                            count
                        )
                    }
                    DisconnectConfigurationReason::CantResolveHost(host) => {
                        log::warn!("can't resolve server host in transfer packet: {}", host)
                    }
                    DisconnectConfigurationReason::DialogError(e) => {
                        log::warn!("dialog error: {:?}", e)
                    }
                    DisconnectConfigurationReason::Timeout => {
                        log::debug!("disconnect because timeout at configuration state")
                    }
                }
                is_online_mode = Some(false);
                is_whitelist = Some(false);
                Self {
                    is_whitelist,
                    is_online_mode,
                    offline_auth,
                    connection_result: pb::ConnectionResult::ConfigurationDisconnect,
                    player_name,
                }
            }
            ConnectionResult::DisconnectAtPlay { data, reason } => {
                match reason {
                    DisconnectPlayReason::DisconnectByServer { .. } => {}
                    DisconnectPlayReason::DialogError(e) => {
                        log::info!("dialog error: {:?}", e)
                    }
                }
                is_online_mode = Some(false);
                is_whitelist = Some(false);
                offline_auth = match data.commands.len() {
                    0 => None,
                    _ => Some(data.commands.iter().any(|c| c == "login")),
                };
                Self {
                    is_whitelist,
                    is_online_mode,
                    offline_auth,
                    connection_result: pb::ConnectionResult::PlayDisconnect,
                    player_name,
                }
            }
            ConnectionResult::Successful { data } => {
                if connection_method > ConnectionMethod::LoginIfNoMsg {
                    is_online_mode = Some(false);
                    is_whitelist = Some(false);
                }
                offline_auth = match data.commands.len() {
                    0 => None,
                    _ => Some(data.commands.iter().any(|c| c == "login")),
                };
                Self {
                    is_whitelist,
                    is_online_mode,
                    offline_auth,
                    connection_result: pb::ConnectionResult::Successful,
                    player_name,
                }
            }
        }
    }
}
fn check_online_mode_by_player_list(data: &ServerData) -> Option<bool> {
    let mut filtered_players = data
        .players
        .iter()
        .filter(|r| r.player.online_type != OnlineType::Anonymous)
        .peekable();
    if filtered_players.peek().is_none() {
        None
    } else {
        Some(filtered_players.all(|r| r.player.online_type == OnlineType::Online))
    }
}
