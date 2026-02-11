use std::sync::Arc;

use mdns_sd::ServiceDaemon;
use reqwest::blocking::Client;

use crate::{
    keylight_repository::KeylightRepository,
    ui::{KeylightUI, Repository},
};

mod keylight;
mod keylight_repository;
mod ui;

fn main() {
    create_gui();
}

fn create_gui() {
    gtk::init().unwrap();

    let mdns = ServiceDaemon::new().unwrap();
    let reqwest = Client::new();
    let keylight_repository: Arc<dyn Repository> = Arc::new(KeylightRepository::new(mdns, reqwest));
    let ui = KeylightUI::new(&keylight_repository);
    ui.show();

    gtk::main();
}
