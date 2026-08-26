use crate::analyze::ExtraServerData;
use crate::mc_proto_server_data::gen_protobuf_server_data;
use data_core::proto::scanner as pb;
use mc_protocol::dialog::{
    ConnectionMethod, ConnectionResult, ConnectionSettings, ServerDialog, ServerDst,
};
use mc_protocol::types::Player;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

pub async fn rescan(
    job_id: u32,
    targets: Vec<pb::RescanTarget>,
    max_connections: usize,
    method: ConnectionMethod,
) -> Result<mpsc::Receiver<pb::RescanUpdate>, tonic::Status> {
    let (tx, rx) = mpsc::channel(256);

    let mut join_set = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(max_connections));

    for target in targets {
        let dst = ServerDst::from_ip_and_port(
            IpAddr::try_from(
                target
                    .ip
                    .ok_or(tonic::Status::invalid_argument("ip is expected"))?,
            )?,
            target.port as u16,
        );
        let semaphore_clone = semaphore.clone();

        join_set.spawn(async move {
            let _permit = semaphore_clone.acquire_owned().await.unwrap();
            parse_server(dst, target.player_name, method).await
        });
    }
    tokio::spawn(async move {
        let mut checked = 0;

        // send job_id first

        let job_id_msg = pb::RescanUpdate {
            job_id,
            checked,
            server_data: None,
        };
        if tx.send(job_id_msg).await.is_err() {
            return;
        }
        while let Some(res) = join_set.join_next().await {
            checked += 1;
            match res {
                Ok(Some(data)) => {
                    let update = pb::RescanUpdate {
                        job_id,
                        checked,
                        server_data: Some(data),
                    };
                    if tx.send(update).await.is_err() {
                        return;
                    };
                }
                Ok(None) => {}
                Err(e) => {
                    if e.is_panic() {
                        panic!("A parsing task panicked: {}", e);
                    }
                }
            }
        }
    });
    Ok(rx)
}

async fn parse_server(
    dst: ServerDst,
    player_name: Option<String>,
    conn_method: ConnectionMethod,
) -> Option<pb::McServerData> {
    let player = match player_name {
        None => Player::random_like_offline(),
        Some(name) => Player::from_offline_name(name).unwrap(),
    };
    let conn_settings = ConnectionSettings {
        conn_method,
        ..Default::default()
    };
    let dialog = ServerDialog::new(dst.clone(), player.clone());
    let result = dialog.connect(conn_settings).await;

    log::trace!("[{}] result: {:?}", dst.socket_addr, result);

    if let ConnectionResult::NoHandshake(e) = &result {
        log::trace!("[{}] can't handshake: {e}", dst.socket_addr);
        return None;
    }

    log::trace!("[{}] parsed", dst.socket_addr);
    let extra_data = ExtraServerData::parse(player, conn_method, &result);
    let data = result.get_data().unwrap(); // data is missing only in NoHandshake, which we check before
    Some(gen_protobuf_server_data(dst, data, extra_data))
}
