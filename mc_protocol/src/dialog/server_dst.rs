use crate::connection::{McConnection, McConnectionError};
use hickory_resolver::proto::rr::RData;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
#[error("can't resolve domain {host}")]
pub struct CantResolveHost {
    host: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServerDst {
    pub server_name: String,
    pub socket_addr: SocketAddr,
}

impl ServerDst {
    pub fn from_ip_and_port(ip: IpAddr, port: u16) -> Self {
        Self {
            server_name: ip.to_string(),
            socket_addr: SocketAddr::from((ip, port)),
        }
    }
    pub async fn from_host_and_port(host: String, port: u16) -> Result<Self, CantResolveHost> {
        match IpAddr::from_str(&host) {
            Ok(ip) => {
                return Ok(Self {
                    server_name: host,
                    socket_addr: SocketAddr::from((ip, port)),
                });
            }
            Err(_) => {}
        };

        Ok(Self {
            server_name: host.clone(),
            socket_addr: get_socket_from_host_and_port(&host, port)
                .await
                .ok_or(CantResolveHost { host })?,
        })
    }
    pub async fn from_like_mc(
        target: String,
        port: Option<u16>,
        resolver: &hickory_resolver::TokioResolver,
    ) -> Result<Self, CantResolveHost> {
        match IpAddr::from_str(&target) {
            // if we get ip, try to get domain record first
            Ok(ip) => {
                let domain = get_domain_from_ip(ip, resolver).await;
                let (server_name, port) = match domain {
                    None => (ip.to_string(), port.unwrap_or(25565)),
                    Some(domain) => {
                        let srv_data = get_srv_record(&domain, resolver).await;
                        if let Some(data) = srv_data {
                            (data.0, port.unwrap_or(data.1))
                        } else {
                            (domain, port.unwrap_or(25565))
                        }
                    }
                };
                Ok(Self {
                    server_name,
                    socket_addr: SocketAddr::from((ip, port)),
                })
            }
            // if we get probably domain
            Err(_) => {
                let srv_data = get_srv_record(&target, resolver).await;
                let (server_name, port) = if let Some(data) = srv_data {
                    (data.0, port.unwrap_or(data.1))
                } else {
                    (target.clone(), port.unwrap_or(25565))
                };
                Ok(Self {
                    server_name,
                    socket_addr: get_socket_from_host_and_port(&target, port)
                        .await
                        .ok_or(CantResolveHost { host: target })?,
                })
            }
        }
    }

    pub async fn make_conn(
        &self,
        protocol: i32,
        read_packet_timeout: Duration,
    ) -> Result<McConnection, McConnectionError> {
        McConnection::new(self.socket_addr, protocol, read_packet_timeout).await
    }
}
async fn get_srv_record(
    host: &str,
    resolver: &hickory_resolver::TokioResolver,
) -> Option<(String, u16)> {
    let query = format!("_minecraft._tcp.{host}");
    let dns_response = match resolver.srv_lookup(query).await {
        Ok(v) => v,
        Err(_) => return None,
    };
    let record = dns_response
        .answers()
        .iter()
        .map(|record| &record.data)
        .filter_map(|rdata| match rdata {
            RData::SRV(srv) => Some(srv),
            _ => None,
        })
        .next();

    record.map(|v| (v.target.to_string(), v.port))
}

async fn get_socket_from_host_and_port(host: &str, port: u16) -> Option<SocketAddr> {
    let addrs = tokio::net::lookup_host((host, port)).await;
    addrs.ok().and_then(|mut i| i.next())
}
async fn get_domain_from_ip(
    ip: IpAddr,
    resolver: &hickory_resolver::TokioResolver,
) -> Option<String> {
    let dns_response = match resolver.reverse_lookup(ip).await {
        Ok(v) => v,
        Err(_) => {
            return None;
        }
    };
    let record = dns_response
        .answers()
        .iter()
        .map(|record| &record.data)
        .filter_map(|rdata| match rdata {
            RData::PTR(ptr) => Some(ptr),
            _ => None,
        })
        .next();

    record.map(|v| v.0.to_string())
}
