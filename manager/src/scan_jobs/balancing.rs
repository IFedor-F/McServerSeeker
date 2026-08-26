use crate::database::ParsedForSqlServerData;
use crate::scan_jobs::worker::WorkerError;
use crate::scan_jobs::{ManagerTask, Worker};
use data_core::api::manager::{
    DiscoverJobProgress, DiscoverRequest, JobProgress, RescanJobProgress, RescanRequest,
    WorkerJobReq,
};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network, NetworkSize};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Balancer {
    workers: Vec<Arc<Worker>>,
}

impl Balancer {
    pub fn new(workers: Vec<Arc<Worker>>) -> Self {
        Self { workers }
    }
    pub async fn run_work(
        self,
        job_task: ManagerTask,
        db_queue: mpsc::Sender<ParsedForSqlServerData>,
    ) -> Result<(), WorkerError> {
        let workers_count = self.workers.len();
        let cancel_token = job_task.cancellation_token;
        let progress = job_task.progress;
        let (tx_stats, mut rx_stats) = mpsc::channel(16);
        let mut worker_futures = FuturesUnordered::new();
        let (requests, weights) = divide_job_request(job_task.req.job_request, &self.workers);
        for (id, (worker, req)) in self.workers.iter().zip(requests).enumerate() {
            worker_futures.push(worker.execute(
                id,
                req,
                tx_stats.clone(),
                db_queue.clone(),
                cancel_token.clone(),
            ));
        }
        let mut updates_vec = vec![JobProgress::default(); workers_count];

        loop {
            tokio::select! {
                Some(worker_result) = worker_futures.next(), if !worker_futures.is_empty() => {
                    if let Err(e) = worker_result {
                        return Err(e);
                    }
                }

                Some(new_progress) = rx_stats.recv() => {
                    updates_vec[new_progress.id] = new_progress.data;
                    let mut current_progress = progress.write().await;
                    *current_progress = sum_updates(&updates_vec, &weights);
                }
                else => {
                    break;
                }
            }
        }
        Ok(())
    }
}

fn divide_job_request(req: WorkerJobReq, workers: &[Arc<Worker>]) -> (Vec<WorkerJobReq>, Vec<f32>) {
    // uses inverse proportion: less load = more weight
    let weights: Vec<f32> = workers
        .into_iter()
        .map(|w| 1.0 / (w.load.get_load() as f32 + 1.0))
        .collect();

    let total_weight: f32 = weights.iter().sum();

    // calculate the exact percentage (share) for each worker
    let shares: Vec<f32> = weights.iter().map(|w| w / total_weight).collect();

    match req {
        WorkerJobReq::Rescan(rescan_req) => (split_rescan(rescan_req, &shares), shares),
        WorkerJobReq::Discover(discover_req) => (split_discover(discover_req, &shares), shares),
    }
}

fn split_rescan(req: RescanRequest, shares: &[f32]) -> Vec<WorkerJobReq> {
    let total_targets = req.targets.len();
    let mut requests = Vec::with_capacity(shares.len());
    let mut targets_iter = req.targets.into_iter();

    for (i, &share) in shares.iter().enumerate() {
        // Last worker takes all remaining targets to avoid floating point rounding loss
        if i == shares.len() - 1 {
            let remaining: Vec<_> = targets_iter.collect();
            requests.push(WorkerJobReq::Rescan(RescanRequest {
                method: req.method.clone(),
                rate: req.rate,
                targets: remaining,
            }));
            break;
        }

        let count = (total_targets as f32 * share).round() as usize;
        let worker_targets: Vec<_> = targets_iter.by_ref().take(count).collect();
        requests.push(WorkerJobReq::Rescan(RescanRequest {
            method: req.method.clone(),
            rate: req.rate,
            targets: worker_targets,
        }));
    }

    requests
}

fn split_discover(req: DiscoverRequest, shares: &[f32]) -> Vec<WorkerJobReq> {
    let total_ips: u128 = req.targets.iter().map(network_size).sum();
    let mut requests = Vec::with_capacity(shares.len());

    let mut targets_queue = req.targets.clone();

    for (i, &share) in shares.iter().enumerate() {
        // Give all remaining networks to the last worker
        if i == shares.len() - 1 {
            requests.push(WorkerJobReq::Discover(DiscoverRequest {
                targets: targets_queue.clone(),
                excludes: req.excludes.clone(),
                ports: req.ports.clone(),
                port_ranges: req.port_ranges.clone(),
                rate: req.rate,
                method: req.method.clone(),
            }));
            break;
        }

        let mut target_ip_count = (total_ips as f32 * share).round() as u128;
        let mut worker_targets = Vec::new();

        while target_ip_count > 0 && !targets_queue.is_empty() {
            let net = targets_queue.remove(0); // Pop from the front
            let size = network_size(&net);

            if size <= target_ip_count {
                // Whole subnet fits into this worker's 'quota'
                worker_targets.push(net);
                target_ip_count = target_ip_count.saturating_sub(size);
            } else {
                // subnet is too big -> split it in half
                if let Some([sub1, sub2]) = split_network_in_half(&net) {
                    // put the halves back into the front of the queue to process next
                    targets_queue.insert(0, sub2);
                    targets_queue.insert(0, sub1);
                } else {
                    // if we can't split it (/32, /128), just give it to the worker
                    worker_targets.push(net);
                    target_ip_count = target_ip_count.saturating_sub(size);
                }
            }
        }

        requests.push(WorkerJobReq::Discover(DiscoverRequest {
            targets: worker_targets,
            excludes: req.excludes.clone(),
            ports: req.ports.clone(),
            port_ranges: req.port_ranges.clone(),
            rate: req.rate,
            method: req.method.clone(),
        }));
    }

    requests
}

fn network_size(net: &IpNetwork) -> u128 {
    match net.size() {
        NetworkSize::V4(size) => size as u128,
        NetworkSize::V6(size) => size,
    }
}

fn split_network_in_half(net: &IpNetwork) -> Option<[IpNetwork; 2]> {
    match net {
        IpNetwork::V4(v4_net) => {
            let prefix = v4_net.prefix();
            if prefix >= 32 {
                return None; // Cannot split a single IP
            }

            let new_prefix = prefix + 1;
            let ip_u32 = u32::from(v4_net.network());

            // Calculate the step size for the new subnet
            let step = 1u32 << (32 - new_prefix);

            let sub1 = Ipv4Network::new(Ipv4Addr::from(ip_u32), new_prefix)
                .expect("valid ip is expected by math");
            let sub2 = Ipv4Network::new(Ipv4Addr::from(ip_u32 + step), new_prefix)
                .expect("valid ip is expected by math");

            Some([IpNetwork::V4(sub1), IpNetwork::V4(sub2)])
        }
        IpNetwork::V6(v6_net) => {
            let prefix = v6_net.prefix();
            if prefix >= 128 {
                return None; // Cannot split a single IPv6
            }

            let new_prefix = prefix + 1;
            let ip_u128 = u128::from(v6_net.network());

            // Calculate the step size for the new subnet
            let step = 1u128 << (128 - new_prefix);

            let sub1 = Ipv6Network::new(Ipv6Addr::from(ip_u128), new_prefix)
                .expect("valid ip is expected by math");
            let sub2 = Ipv6Network::new(Ipv6Addr::from(ip_u128 + step), new_prefix)
                .expect("valid ip is expected by math");

            Some([IpNetwork::V6(sub1), IpNetwork::V6(sub2)])
        }
    }
}

fn sum_updates(values: &[JobProgress], weights: &[f32]) -> JobProgress {
    let values: Vec<_> = values
        .into_iter()
        .filter(|v| !matches!(v, JobProgress::NoData))
        .collect();

    if values.is_empty() {
        return JobProgress::NoData;
    }

    match values[0] {
        JobProgress::NoData => unreachable!(),
        JobProgress::Discover(_) => JobProgress::Discover(sum_discover_update(
            values
                .into_iter()
                .map(|v| match v {
                    JobProgress::Discover(v) => v,
                    _ => {
                        panic!("different values in sum_updates function")
                    }
                })
                .collect(),
            weights,
        )),
        JobProgress::Rescan(_) => JobProgress::Rescan(sum_rescan_update(
            values
                .into_iter()
                .map(|v| match v {
                    JobProgress::Rescan(v) => v,
                    _ => {
                        panic!("different values in sum_updates function")
                    }
                })
                .collect(),
        )),
    }
}

fn sum_rescan_update(values: Vec<&RescanJobProgress>) -> RescanJobProgress {
    RescanJobProgress {
        all: values[0].all,
        checked: values.iter().map(|v| v.checked).sum(),
        successful: values.iter().map(|v| v.successful).sum(),
    }
}

fn sum_discover_update(values: Vec<&DiscoverJobProgress>, weights: &[f32]) -> DiscoverJobProgress {
    DiscoverJobProgress {
        scanned_progress: values
            .iter()
            .zip(weights)
            .map(|(v, &w)| v.scanned_progress * w)
            .sum(),
        founded: values.iter().map(|v| v.founded).sum(),
        parsing_now: values.iter().map(|v| v.parsing_now).sum(),
        successful: values.iter().map(|v| v.successful).sum(),
    }
}
