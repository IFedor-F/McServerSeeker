use crate::analyze::ExtraServerData;
use crate::mc_proto_server_data::gen_protobuf_server_data;
use crate::scan::masscan::MasscanBuilder;
use data_core::proto::scanner as pb;
use mc_protocol::dialog::{
    ConnectionMethod, ConnectionResult, ConnectionSettings, ServerDialog, ServerDst,
};
use mc_protocol::types::Player;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

pub fn scan_inet(
    job_id: u32,
    masscan_builder: MasscanBuilder,
    max_connections: usize,
    method: ConnectionMethod,
) -> mpsc::Receiver<pb::DiscoverUpdate> {
    let (tx_out, rx_out) = mpsc::channel(200);
    tokio::spawn(async move {
        let (mut rx_results, mut rx_progress) = masscan_builder.run().await;
        let mut join_set = JoinSet::new();
        let semaphore = Arc::new(Semaphore::new(max_connections));

        let mut stats = pb::ScanStats {
            scanned_progress: 0.0,
            founded: 0,
            parsing_now: 0,
            successful: 0,
        };

        let mut rx_results_open = true;
        let mut rx_progress_open = true;

        // send job_id first
        let job_id_msg = pb::DiscoverUpdate {
            job_id,
            stats: Some(stats),
            server_data: None,
        };
        if tx_out.send(job_id_msg).await.is_err() {
            return;
        }

        loop {
            if !rx_results_open && !rx_progress_open && join_set.is_empty() {
                break;
            }

            tokio::select! {
                res = rx_results.recv(), if rx_results_open => {
                    match res {
                        Some(result) => {
                            log::trace!("[job {}] found open port: {}:{}", job_id, result.ip, result.port);
                            stats.founded += 1;
                            stats.parsing_now += 1;

                            let dst = ServerDst::from_ip_and_port(result.ip, result.port);
                            let permit = semaphore.clone().acquire_owned().await.unwrap();

                            join_set.spawn(async move {
                                let _permit = permit;
                                parse_server(dst, method).await
                            });

                            let update = pb::DiscoverUpdate {
                                job_id,
                                stats: Some(stats),
                                server_data: None
                            };
                            if tx_out.send(update).await.is_err() {
                                break
                            }
                        }
                        None => {
                            rx_results_open = false;
                        }
                    }
                }

                res = rx_progress.recv(), if rx_progress_open => {
                    match res {
                        Some(pct) => {
                            stats.scanned_progress = pct;
                            let update = pb::DiscoverUpdate {
                                job_id,
                                stats: Some(stats),
                                server_data: None
                            };
                            if tx_out.send(update).await.is_err() {
                                break
                            }
                        }
                        None => {
                            rx_progress_open = false;
                        }
                    }
                }

                Some(res) = join_set.join_next(), if !join_set.is_empty() => {
                    stats.parsing_now -= 1;

                    match res {
                        Ok(Some(data)) => {
                            stats.successful += 1;
                            let update = pb::DiscoverUpdate {
                                job_id,
                                stats: Some(stats),
                                server_data: Some(data)
                            };
                            let _ = tx_out.send(update).await;
                        }
                        Ok(None) => {
                        }
                        Err(e) => {
                            if e.is_cancelled() {
                                return
                            }
                            panic!("parsing or masscan task panicked: {}", e);
                        }
                    }
                }
            }
        }
    });
    rx_out
}

pub async fn parse_server(
    dst: ServerDst,
    conn_method: ConnectionMethod,
) -> Option<pb::McServerData> {
    let player = Player::random_like_offline();

    let dialog = ServerDialog::new(dst.clone(), player.clone());
    let conn_settings = ConnectionSettings {
        conn_method,
        ..Default::default()
    };

    let result = dialog.connect(conn_settings).await;
    log::trace!("[{}] result: {:?}", dst.socket_addr, result);
    match result {
        ConnectionResult::NoHandshake(e) => {
            log::trace!("[{}] can't handshake: {e}", dst.socket_addr);
            return None;
        }
        _ => {}
    }
    log::trace!("[{}] parsed", dst.socket_addr);
    let extra_data = ExtraServerData::parse(player, conn_method, &result);
    let data = result.get_data().unwrap(); // has data because we check this is not 'NoHandshake'
    Some(gen_protobuf_server_data(dst, data, extra_data))
}
