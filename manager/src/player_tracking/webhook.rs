use crate::player_tracking::FoundedPlayer;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use log;
use reqwest::Client;
use serde::Serialize;
use std::net::IpAddr;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

// The main message payload sent to Discord
#[derive(Serialize)]
struct DiscordMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,

    embeds: Vec<Embed>,
}

#[derive(Serialize)]
struct Embed {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<EmbedField>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<EmbedFooter>,
}

// A field inside the embed (acts like a key-value row)
#[derive(Serialize)]
struct EmbedField {
    name: String,
    value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    inline: Option<bool>,
}

#[derive(Serialize)]
struct EmbedFooter {
    text: String,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct WebHook {
    url: Url,
}
impl WebHook {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
    pub async fn send_players<U>(&self, records: U)
    where
        U: IntoIterator<Item = FoundedPlayer>,
    {
        let client = Client::new();

        let rec_map_iter = records
            .into_iter()
            .map(|r| ((r.uuid, r.name), (r.ip, r.port, r.last_seen)))
            .into_group_map();

        for (pl, serv) in rec_map_iter {
            let payload = format_msg(pl, serv);
            let response = client.post(self.url.clone()).json(&payload).send().await;
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        log::debug!("embed was set to {}", self.url);
                    } else {
                        log::warn!(
                            "embed was failed to send: HTTP {}, embed url: {}",
                            resp.status(),
                            self.url
                        );
                    }
                }
                Err(_) => {
                    log::warn!("embed was failed to send, embed url: {}", self.url);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await // to avoid discord send limit
        }
    }
}

fn format_msg(pl: (Uuid, String), serv: Vec<(IpAddr, i32, DateTime<Utc>)>) -> DiscordMessage {
    let servers_count = serv.len();

    let servers_fields: Vec<EmbedField> = serv
        .into_iter()
        .take(10)
        .map(|v| EmbedField {
            name: format!("{}:{}", v.0, v.1),
            value: format!("<t:{}:F>", v.2.timestamp()),
            inline: None,
        })
        .collect();

    let footer = if servers_count > 10 {
        Some(EmbedFooter {
            text: format!("And also other {} servers", servers_count - 10),
        })
    } else {
        None
    };

    let server_embed = Embed {
        title: Some("Servers".to_string()),
        description: None,
        color: Some(0x7BCBF0),
        fields: Some(servers_fields),
        footer,
    };

    let main_embed = Embed {
        title: Some("Found player".to_string()),
        description: Some(format!("{} was found on {} servers", pl.1, servers_count)),
        color: Some(0x4FFF72),
        fields: Some(vec![
            EmbedField {
                name: "Name".to_string(),
                value: pl.1,
                inline: None,
            },
            EmbedField {
                name: "UUID".to_string(),
                value: pl.0.to_string(),
                inline: None,
            },
        ]),
        footer: None,
    };

    DiscordMessage {
        content: None,
        username: None,
        embeds: vec![main_embed, server_embed],
    }
}
