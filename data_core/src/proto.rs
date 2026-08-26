pub mod scanner {
    use crate::api;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use tonic::Status;

    tonic::include_proto!("scanner");

    pub struct UnexpectedUnknown;

    impl TryFrom<IpAddr> for std::net::IpAddr {
        type Error = Status;

        fn try_from(value: IpAddr) -> Result<Self, Self::Error> {
            let IpAddr {
                ip_version,
                address,
            } = value;
            let ip_version = IpVersion::try_from(ip_version).map_err(|e| {
                Status::invalid_argument(format!("Invalid ip version in IpAddr: {}", e.0))
            })?;
            match ip_version {
                IpVersion::V4 => {
                    let octets: [u8; 4] = address
                        .try_into()
                        .map_err(|_| Status::invalid_argument("Invalid ip bytes"))?;
                    Ok(Self::V4(Ipv4Addr::from_octets(octets)))
                }
                IpVersion::V6 => {
                    let octets: [u8; 16] = address
                        .try_into()
                        .map_err(|_| Status::invalid_argument("Invalid ip bytes"))?;
                    Ok(Self::V6(Ipv6Addr::from_octets(octets)))
                }
            }
        }
    }
    impl From<std::net::IpAddr> for IpAddr {
        fn from(value: std::net::IpAddr) -> Self {
            let (ip_version, octets) = match value {
                std::net::IpAddr::V4(ip) => (IpVersion::V4, ip.octets().to_vec()),
                std::net::IpAddr::V6(ip) => (IpVersion::V6, ip.octets().to_vec()),
            };
            Self {
                ip_version: ip_version as i32,
                address: octets,
            }
        }
    }
    impl TryFrom<IpPrefix> for ipnetwork::IpNetwork {
        type Error = Status;
        fn try_from(prefix: IpPrefix) -> Result<Self, Status> {
            let prefix_len: u8 = prefix
                .prefix_length
                .try_into()
                .map_err(|_| Status::invalid_argument("Prefix length must fit in u8"))?;
            let ip = std::net::IpAddr::try_from(prefix.ip_addr.ok_or(
                Status::invalid_argument("Ip field is expected in ip prefix"),
            )?)?;
            Ok(Self::new(ip, prefix_len)
                .map_err(|_| Status::invalid_argument("Invalid IpPrefix"))?)
        }
    }
    impl From<ipnetwork::IpNetwork> for IpPrefix {
        fn from(value: ipnetwork::IpNetwork) -> Self {
            let ip_addr = Some(IpAddr::from(value.ip()));
            let prefix_length = value.prefix() as u32;
            Self {
                ip_addr,
                prefix_length,
            }
        }
    }
    impl From<ScanMethod> for mc_protocol::dialog::ConnectionMethod {
        fn from(value: ScanMethod) -> Self {
            match value {
                ScanMethod::OnlyHandshake => mc_protocol::dialog::ConnectionMethod::OnlyHandshake,
                ScanMethod::LoginIfNoMsg => mc_protocol::dialog::ConnectionMethod::LoginIfNoMsg,
                ScanMethod::JoinIfEmpty => mc_protocol::dialog::ConnectionMethod::JoinIfEmpty,
                ScanMethod::Join => mc_protocol::dialog::ConnectionMethod::Join,
            }
        }
    }

    impl From<mc_protocol::dialog::ConnectionMethod> for ScanMethod {
        fn from(value: mc_protocol::dialog::ConnectionMethod) -> Self {
            match value {
                mc_protocol::dialog::ConnectionMethod::OnlyHandshake => ScanMethod::OnlyHandshake,
                mc_protocol::dialog::ConnectionMethod::LoginIfNoMsg => ScanMethod::LoginIfNoMsg,
                mc_protocol::dialog::ConnectionMethod::JoinIfEmpty => ScanMethod::JoinIfEmpty,
                mc_protocol::dialog::ConnectionMethod::Join => ScanMethod::Join,
            }
        }
    }

    impl From<Difficulty> for Option<mc_protocol::types::Difficulty> {
        fn from(value: Difficulty) -> Option<mc_protocol::types::Difficulty> {
            match value {
                Difficulty::Peaceful => Some(mc_protocol::types::Difficulty::Peaceful),
                Difficulty::Easy => Some(mc_protocol::types::Difficulty::Easy),
                Difficulty::Normal => Some(mc_protocol::types::Difficulty::Normal),
                Difficulty::Hard => Some(mc_protocol::types::Difficulty::Hard),
                Difficulty::UnknownDifficulty => None,
            }
        }
    }

    impl From<mc_protocol::types::Difficulty> for Difficulty {
        fn from(value: mc_protocol::types::Difficulty) -> Self {
            match value {
                mc_protocol::types::Difficulty::Peaceful => Difficulty::Peaceful,
                mc_protocol::types::Difficulty::Easy => Difficulty::Easy,
                mc_protocol::types::Difficulty::Normal => Difficulty::Normal,
                mc_protocol::types::Difficulty::Hard => Difficulty::Hard,
            }
        }
    }

    impl From<GameMode> for Option<mc_protocol::types::GameMode> {
        fn from(value: GameMode) -> Option<mc_protocol::types::GameMode> {
            match value {
                GameMode::Survival => Some(mc_protocol::types::GameMode::Survival),
                GameMode::Creative => Some(mc_protocol::types::GameMode::Creative),
                GameMode::Adventure => Some(mc_protocol::types::GameMode::Adventure),
                GameMode::Spectator => Some(mc_protocol::types::GameMode::Spectator),
                GameMode::UnknownGameMode => None,
            }
        }
    }
    impl From<mc_protocol::types::GameMode> for GameMode {
        fn from(value: mc_protocol::types::GameMode) -> Self {
            match value {
                mc_protocol::types::GameMode::Survival => GameMode::Survival,
                mc_protocol::types::GameMode::Creative => GameMode::Creative,
                mc_protocol::types::GameMode::Adventure => GameMode::Adventure,
                mc_protocol::types::GameMode::Spectator => GameMode::Spectator,
            }
        }
    }

    impl From<ResourcePack> for mc_protocol::dialog::ResourcePack {
        fn from(value: ResourcePack) -> Self {
            mc_protocol::dialog::ResourcePack {
                url: value.url,
                hash: value.hash,
                forced: value.forced,
            }
        }
    }
    impl From<mc_protocol::dialog::ResourcePack> for ResourcePack {
        fn from(value: mc_protocol::dialog::ResourcePack) -> Self {
            ResourcePack {
                url: value.url,
                hash: value.hash,
                forced: value.forced,
            }
        }
    }
    impl From<McMod> for mc_protocol::states::status::s2c::status_response::ForgeMod {
        fn from(value: McMod) -> Self {
            mc_protocol::states::status::s2c::status_response::ForgeMod {
                mod_id: value.mod_id,
                version: value.version,
            }
        }
    }

    impl From<mc_protocol::states::status::s2c::status_response::ForgeMod> for McMod {
        fn from(value: mc_protocol::states::status::s2c::status_response::ForgeMod) -> Self {
            McMod {
                mod_id: value.mod_id,
                version: value.version,
            }
        }
    }
    impl From<api::manager::PortRange> for PortRange {
        fn from(value: api::manager::PortRange) -> Self {
            Self {
                min: value.min as u32,
                max: value.max as u32,
            }
        }
    }

    impl From<PortRange> for api::manager::PortRange {
        fn from(value: PortRange) -> Self {
            Self {
                min: value.min as u16,
                max: value.max as u16,
            }
        }
    }
    impl From<api::manager::DiscoverRequest> for DiscoverRequest {
        fn from(value: api::manager::DiscoverRequest) -> Self {
            Self {
                targets: value.targets.into_iter().map(IpPrefix::from).collect(),
                excludes: value.excludes.into_iter().map(IpPrefix::from).collect(),
                ports: value.ports.into_iter().map(|v| v as u32).collect(),
                port_ranges: value.port_ranges.into_iter().map(PortRange::from).collect(),
                rate: value.rate,
                method: value.method as i32,
            }
        }
    }

    impl From<api::manager::RescanTarget> for RescanTarget {
        fn from(value: api::manager::RescanTarget) -> Self {
            Self {
                ip: Some(IpAddr::from(value.ip)),
                port: value.port as u32,
                player_name: value.player_name,
            }
        }
    }
    impl From<api::manager::RescanRequest> for RescanRequest {
        fn from(value: api::manager::RescanRequest) -> Self {
            Self {
                method: value.method as i32,
                rate: value.rate,
                targets: value.targets.into_iter().map(RescanTarget::from).collect(),
            }
        }
    }
}
