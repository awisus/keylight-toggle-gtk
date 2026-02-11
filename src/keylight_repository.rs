use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use gtk::gio::spawn_blocking;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::{
    keylight::Keylight,
    ui::{self},
};

const SERVICE_TYPE_NAME: &str = "_elg._tcp.local.";

pub struct KeylightRepository {
    mdns: Arc<ServiceDaemon>,
    reqwest: Arc<Client>,
}

impl KeylightRepository {
    pub fn new(mdns: ServiceDaemon, reqwest: Client) -> Self {
        Self {
            mdns: Arc::new(mdns),
            reqwest: Arc::new(reqwest),
        }
    }
}

#[async_trait]
impl ui::Repository for KeylightRepository {
    async fn list(&self) -> Result<Vec<Keylight>, String> {
        let mdns = self.mdns.clone();
        let reqwest = self.reqwest.clone();

        spawn_blocking(move || {
            let mut lights = Vec::new();

            let receiver = mdns.browse(SERVICE_TYPE_NAME).unwrap();
            let timeout = std::time::Instant::now() + Duration::from_secs(2);

            while let Ok(event) = receiver.recv_timeout(Duration::from_millis(200)) {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let ip = info.get_addresses_v4().iter().next().cloned();
                        let port = info.get_port();

                        if let Some(ip) = ip {
                            let base_url = format!("http://{}:{}", ip, port);
                            let url = format!("{}/elgato/lights", base_url);

                            if let Ok(resp) = reqwest.get(&url).send() {
                                if let Ok(state) = resp.json::<LightsState>() {
                                    if let Some(light_state) = state.lights.first() {
                                        let on = light_state.on == 1;
                                        lights.push(Keylight::new(&base_url, on));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                if std::time::Instant::now() > timeout {
                    break;
                }
            }

            lights
        })
        .await
        // TODO: get real error
        .map_err(|_| "error".to_string())
    }

    async fn switch(&self, keylights: Vec<Keylight>, on: bool) -> Result<(), String> {
        for keylight in keylights {
            let url = format!("{}/elgato/lights", keylight.url);
            let body = LightsState {
                lights: vec![LightState {
                    on: if on { 1 } else { 0 },
                }],
            };
            let response = self.reqwest.put(&url).json(&body).send();
            if let Err(error) = response {
                return Err(error.to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LightsState {
    lights: Vec<LightState>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LightState {
    on: u8,
}
