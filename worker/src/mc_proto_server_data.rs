use crate::analyze::ExtraServerData;
use data_core::proto::scanner as pb;
use mc_protocol::dialog::{PlayerRecord, ServerData, ServerDst};
use mc_protocol::types::player::OnlineType;
use std::collections::HashSet;

pub fn gen_protobuf_server_data(
    dst: ServerDst,
    server_data: ServerData,
    extra_server_data: ExtraServerData,
) -> pb::McServerData {
    let ServerData {
        description,
        protocol,
        version_name,
        online_players,
        max_players,
        players,
        enforces_secure_chat,
        no_chat_reports,
        mods,
        brand,
        links,
        code_of_conduct,
        features,
        resource_pack,
        channels,
        registered_channels,
        commands,
        hashed_seed,
        gamemode,
        difficulty,
        dimension,
        view_distance,
        simulation_distance,
        is_hardcore,
        reduced_debug_info,
        do_limited_crafting,
        is_flat,
    } = server_data;

    let ExtraServerData {
        is_whitelist,
        is_online_mode,
        offline_auth,
        connection_result,
        player_name,
    } = extra_server_data;

    let ip = dst.socket_addr.ip();
    let domain = if dst.server_name == dst.socket_addr.ip().to_string() {
        None
    } else {
        Some(dst.server_name)
    };
    let channels = (&channels | &registered_channels).into_iter().collect();

    pb::McServerData {
        ip: Some(pb::IpAddr::from(ip)),
        port: dst.socket_addr.port() as u32,
        domain,
        connection_result: connection_result as i32,
        description,
        protocol,
        version_name,
        online_players,
        max_players,
        players: to_proto_players(players),
        enforces_secure_chat,
        no_chat_reports,
        mods: mods.into_iter().map(pb::McMod::from).collect(),

        channels,
        brand,
        links,
        code_of_conduct,
        features,
        resource_pack: resource_pack.and_then(|v| Some(v.into())),
        hashed_seed,
        gamemode: gamemode.and_then(|v| Some(pb::GameMode::from(v) as i32)),
        difficulty: difficulty.and_then(|v| Some(pb::Difficulty::from(v) as i32)),
        dimension,
        view_distance: view_distance.and_then(|v| Some(v as u32)),
        simulation_distance: simulation_distance.and_then(|v| Some(v as u32)),
        is_hardcore,
        reduced_debug_info,
        do_limited_crafting,
        is_flat,
        commands,

        is_whitelist,
        is_online_mode,
        offline_auth,
        player_name: Some(player_name),
    }
}
fn to_proto_players(players: HashSet<PlayerRecord>) -> Vec<pb::PlayerRecord> {
    players
        .into_iter()
        .filter_map(|x| {
            if x.player.online_type == OnlineType::Anonymous {
                None
            } else {
                Some(pb::PlayerRecord {
                    name: x.player.name,
                    uuid: x.player.uuid.as_bytes().to_vec(),
                    is_online: x.player.online_type == OnlineType::Online,
                    gamemode: x.game_mode.and_then(|x| Some(pb::GameMode::from(x) as i32)),
                    ping: x.ping,
                })
            }
        })
        .collect()
}
