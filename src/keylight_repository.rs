use std::{collections::HashSet, pin::Pin, sync::Arc, time::Duration};

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    keylight::Keylight,
    ui::{self},
};

use futures_util::{Stream, future, stream::StreamExt};
use mdns::{Record, RecordKind, Response};
use std::net::IpAddr;
use tokio::time::timeout;
use anyhow::Result;
use anyhow::Context;

const SERVICE_TYPE_NAME: &str = "_elg._tcp.local";

pub struct KeylightRepository {
    discovery: Arc<dyn Discovery>,
    reqwest: Arc<Client>,
}

impl KeylightRepository {
    pub fn new(discovery: Arc<dyn Discovery>, reqwest: Arc<Client>) -> Self {
        Self {
            discovery,
            reqwest,
        }
    }
}

#[async_trait]
impl ui::Repository for KeylightRepository {
    async fn list(&self) -> Result<Vec<Keylight>> {
        let mut urls = HashSet::new();
        let mut stream = self.discovery.all(SERVICE_TYPE_NAME, Duration::from_millis(200))
            .context("Failed to start mDNS discovery")?;
        while let Ok(response) = timeout(Duration::from_millis(500), stream.next()).await {
            match response {
                Some(Ok(info)) => {
                    if let Some(url) = to_url(&info) {
                        urls.insert(url);
                    }
                }
                Some(Err(error)) => eprintln!("mDNS Paket-Fehler: {}", error),
                None => break,
            }
        }

        let tasks = urls.into_iter().map(|url| {
            let reqwest = self.reqwest.clone();
            async move {
                let state = light_state(&reqwest, &url).await.ok()?;
                Some(Keylight::new(&url, state.lights.first()?.on == 1))
            }
        });
        let lights = future::join_all(tasks).await.into_iter().flatten().collect();

        Ok(lights)
    }

    async fn switch(&self, keylights: Vec<Keylight>, on: bool) -> Result<()> {
        let tasks = keylights.into_iter().map(|keylight| {
            let reqwest = self.reqwest.clone();
            let url = format!("{}/elgato/lights", keylight.url);
            let body = LightsState {
                lights: vec![LightState {
                    on: if on { 1 } else { 0 },
                }],
            };
            async move {
                reqwest.put(&url).json(&body).send().await
            }        
        });
        future::join_all(tasks).await;

        Ok(())
    }
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}

fn to_url(response: &Response) -> Option<String> {
    let ip = response
        .records()
        .filter_map(self::to_ip_addr)
        .next()?
        .to_string();
    let port = response.port()?;
    
    Some(format!("http://{}:{}", ip, port))
}

async fn light_state(reqwest: &Client, base_url: &str) -> Result<LightsState> {
    let url = format!("{}/elgato/lights", base_url);
    let state = reqwest
        .get(&url)
        .send()
        .await?
        .json::<LightsState>()
        .await?;

    Ok(state)
}

pub trait Discovery: Send + Sync {
    fn all(&self, service_name: &str, query_interval: Duration) -> Result<Pin<Box<dyn Stream<Item = Result<Response>> + Send>>>;
}

#[derive(Serialize, Deserialize)]
struct LightsState {
    lights: Vec<LightState>,
}

#[derive(Serialize, Deserialize)]
struct LightState {
    on: u8,
}
