use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub ip: IpAddr,
    pub port: u16,
}

pub struct MasscanBuilder {
    targets: Vec<IpNetwork>,
    excludes: Vec<IpNetwork>,
    ports: Vec<u16>,
    port_ranges: Vec<(u16, u16)>,
    rate: Option<u32>,
}

impl MasscanBuilder {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            excludes: Vec::new(),
            ports: Vec::new(),
            port_ranges: Vec::new(),
            rate: None,
        }
    }

    pub fn target(mut self, target: IpNetwork) -> Self {
        self.targets.push(target);
        self
    }

    pub fn exclude(mut self, exclude: IpNetwork) -> Self {
        self.excludes.push(exclude);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.ports.push(port);
        self
    }
    pub fn port_range(mut self, min: u16, max: u16) -> Self {
        self.port_ranges.push((min, max));
        self
    }

    pub fn rate(mut self, rate: u32) -> Self {
        self.rate = Some(rate);
        self
    }

    pub async fn run(self) -> (mpsc::Receiver<ScanResult>, mpsc::Receiver<f32>) {
        let masscan_command = std::env::var("MASSCAN_COMMAND").unwrap_or("masscan".to_string());
        let extra_args_str = std::env::var("MASSCAN_ARGS").ok();

        let mut cmd = Command::new(masscan_command);

        if let Some(args) = extra_args_str {
            match shlex::split(&args) {
                None => {
                    panic!("failed to build masscan with args form env: {args}")
                }
                Some(parsed_args) => {
                    cmd.args(parsed_args);
                }
            }
        }

        for target in &self.targets {
            cmd.arg(target.to_string());
        }
        for exclude in &self.excludes {
            cmd.arg("--exclude").arg(exclude.to_string());
        }

        let mut p_arg: Vec<String> = Vec::new();
        if self.port_ranges.len() > 0 {
            p_arg.extend(
                self.port_ranges
                    .into_iter()
                    .map(|v| format!("{}-{}", v.0, v.1))
                    .collect::<Vec<String>>(),
            );
        }
        if self.ports.len() > 0 {
            p_arg.extend(self.ports.into_iter().map(|v| v.to_string()));
        }

        if p_arg.len() > 0 {
            cmd.arg("-p").arg(p_arg.join(","));
        }

        if let Some(r) = self.rate {
            cmd.arg("--rate").arg(r.to_string());
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Can't spawn masscan");

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let (tx_results, rx_results) = mpsc::channel::<ScanResult>(1000);
        let (tx_progress, rx_progress) = mpsc::channel::<f32>(100);

        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut buf = Vec::new();

            while let Ok(bytes_read) = reader.read_until(b'\r', &mut buf).await {
                if bytes_read == 0 {
                    break;
                }

                let line = String::from_utf8_lossy(&buf);

                if line.contains("[-] FAIL:") {
                    panic!("Can't spawn masscan:\n{}", line);
                }

                if line.contains("% done") {
                    if let Some(pct_str) = line.split_whitespace().find(|s| s.ends_with('%')) {
                        let clean_str = pct_str.trim_end_matches('%');
                        if let Ok(pct) = clean_str.parse::<f32>() {
                            let _ = tx_progress.send(pct).await;
                        }
                    }
                }
                buf.clear();
            }
        });

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();

            while let Ok(Some(line)) = reader.next_line().await {
                if line.starts_with("Discovered open port") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() == 6 {
                        let port_proto = parts[3];
                        let ip = parts[5].to_string();

                        let pp_parts: Vec<&str> = port_proto.split('/').collect();
                        if pp_parts.len() == 2 {
                            if let Ok(port) = pp_parts[0].parse::<u16>() {
                                let result = ScanResult {
                                    ip: ip.parse().unwrap(),
                                    port,
                                };
                                if tx_results.send(result).await.is_err() {
                                    let _ = child.kill().await;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            let _ = child.wait().await;
        });
        (rx_results, rx_progress)
    }
}
