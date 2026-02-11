use std::{pin::Pin, time::Duration};

use futures_util::{Stream, StreamExt};
use mdns::{Response, discover::{self}};

use crate::keylight_repository::Discovery;

use anyhow::{Result, anyhow};

pub struct MdnsDiscovery;

impl Discovery for MdnsDiscovery {
    fn all(&self, service_name: &str, query_interval: Duration) -> Result<Pin<Box<dyn Stream<Item = Result<Response>> + Send>>> {
        let stream = discover::all(service_name, query_interval)
            .map_err(|error| anyhow!(error))?
            .listen()
            .map(|item| item.map_err(|error| anyhow!(error)));

        Ok(Box::pin(stream))
    }
}
