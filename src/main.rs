use std::env;
use std::process;
use std::sync::Arc;

use clap::Parser;
use reqwest::Client;

use crate::mdns_discovery::MdnsDiscovery;
use crate::{
    keylight_repository::KeylightRepository,
    ui::{KeylightUI, Repository},
};

mod keylight;
mod keylight_repository;
mod ui;
mod mdns_discovery;

#[tokio::main]
async fn main() {
    let app_state = app_state();

    if env::args_os().len() == 1 {
        create_ui(app_state);
        return;
    }

    run_cli(app_state).await;
}

fn create_ui(app_state: Arc<AppState>) {
    gtk::init().unwrap();

    let ui = KeylightUI::new(app_state.keylight_repository.clone());
    ui.show();

    gtk::main();
}

async fn run_cli(app_state: Arc<AppState>) {
    let args = Args::parse();
    let repo = app_state.keylight_repository.clone();

    let lights = match repo.list().await {
        Ok(l) if !l.is_empty() => l,
        Ok(_) => {
            eprintln!("Keine Keylights gefunden");
            process::exit(2);
        }
        Err(_) => {
            eprintln!("Fehler beim Discovern");
            process::exit(1);
        }
    };
    let current = lights[0].on;

    let next = if args.on {
        true
    } else if args.off {
        false
    } else if args.toggle {
        !current
    } else {
        unreachable!();
    };
    if let Err(_) = repo.switch(lights, next).await {
        process::exit(1);
    }
}

fn app_state() -> Arc<AppState> {
    let discovery = MdnsDiscovery {};
    let reqwest = Client::new();

    Arc::new(AppState {
        keylight_repository: Arc::new(KeylightRepository::new(Arc::new(discovery), Arc::new(reqwest)))
    })
}

#[derive(Clone)]
pub struct AppState {
    pub keylight_repository: Arc<KeylightRepository>,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    on: bool,

    #[arg(long)]
    off: bool,

    #[arg(long)]
    toggle: bool,
}
