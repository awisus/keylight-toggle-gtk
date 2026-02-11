use crate::keylight::Keylight;

use std::{cell::RefCell, env, rc::Rc, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use gtk::{Menu, MenuItem, SeparatorMenuItem, prelude::*};
use libappindicator::{AppIndicator, AppIndicatorStatus};

pub struct KeylightUI {
    repository: Arc<dyn Repository>,

    on: Rc<RefCell<Option<bool>>>,
    lights: Rc<RefCell<Vec<Keylight>>>,

    indicator: Rc<RefCell<AppIndicator>>,
    menu: Menu,
    toggle_button: Rc<RefCell<MenuItem>>,
    search_button: Rc<RefCell<MenuItem>>,
    exit_button: MenuItem,
}

impl KeylightUI {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        let mut menu = Menu::new();
        let toggle_button = MenuItem::with_label("Suche...");
        toggle_button.set_sensitive(false);
        menu.append(&toggle_button);
        menu.append(&SeparatorMenuItem::new());
        let search_button = MenuItem::with_label("Suchen");
        menu.append(&search_button);
        let exit_button = MenuItem::with_label("Beenden");
        menu.append(&exit_button);
        let mut indicator = AppIndicator::new("keylight-toggle", &indictor_icon());
        indicator.set_menu(&mut menu);

        let ui = Self {
            repository: repository.clone(),
            on: Rc::new(RefCell::new(None)),
            lights: Rc::new(RefCell::new(Vec::new())),
            indicator: Rc::new(RefCell::new(indicator)),
            menu: menu,
            toggle_button: Rc::new(RefCell::new(toggle_button)),
            search_button: Rc::new(RefCell::new(search_button)),
            exit_button: exit_button,
        };
        ui.setup_callbacks();
        ui.discover();
        ui
    }

    pub fn show(&self) {
        self.indicator
            .borrow_mut()
            .set_status(AppIndicatorStatus::Active);
        self.menu.show_all();
    }

    fn setup_callbacks(&self) {
        let toggle_repo = self.repository.clone();
        let toggle_on = self.on.clone();
        let toggle_lights = self.lights.clone();
        let toggle_toggle = self.toggle_button.clone();
        self.toggle_button.borrow().connect_activate(move |_| {
            let repo = toggle_repo.clone();
            let on = toggle_on.clone();
            let lights = toggle_lights.clone();
            let toggle_button = toggle_toggle.clone();

            toggle_button.borrow_mut().set_sensitive(false);

            glib::MainContext::default().spawn_local(async move {
                let state = *on.borrow_mut();
                if let Some(current) = state {
                    let next = !current;
                    if let Err(err) = repo.switch(lights.borrow().clone(), next).await {
                        eprintln!("{}", &err);
                        return;
                    }

                    *on.borrow_mut() = Some(next);
                    for light in lights.borrow_mut().iter_mut() {
                        light.on = next;
                    }
                    toggle_button.borrow_mut().set_label(&toggle_label(next));
                }

                toggle_button.borrow_mut().set_sensitive(true);
            });
        });

        let search_repo = self.repository.clone();
        let search_on = self.on.clone();
        let search_lights = self.lights.clone();
        let search_toggle = self.toggle_button.clone();
        self.search_button.borrow().connect_activate(move |_| {
            discover_lights(
                search_repo.clone(),
                search_on.clone(),
                search_lights.clone(),
                search_toggle.clone(),
            );
        });

        self.exit_button.connect_activate(|_| {
            gtk::main_quit();
        });
    }

    fn discover(&self) {
        discover_lights(
            self.repository.clone(),
            self.on.clone(),
            self.lights.clone(),
            self.toggle_button.clone(),
        );
    }
}

fn discover_lights(
    repo: Arc<dyn Repository>,
    on: Rc<RefCell<Option<bool>>>,
    lights: Rc<RefCell<Vec<Keylight>>>,
    toggle_button: Rc<RefCell<MenuItem>>,
) {
    glib::MainContext::default().spawn_local(async move {
        let toggle_button = toggle_button.borrow_mut();

        toggle_button.set_label("Suche...");
        toggle_button.set_sensitive(false);

        *on.borrow_mut() = None;
        if let Ok(found_lights) = repo.list().await {
            if let Some(first) = found_lights.first() {
                let is_on = first.on;
                *on.borrow_mut() = Some(is_on);
                *lights.borrow_mut() = found_lights;
                toggle_button.set_label(&toggle_label(is_on));
                toggle_button.set_sensitive(true);
            } else {
                toggle_button.set_label("Keine Keylights");
            }
        } else {
            toggle_button.set_label("Keine Verbindung");
        }
    });
}

fn toggle_label(on: bool) -> String {
    (if on { "Ausschalten" } else { "Einschalten" }).to_string()
}

fn indictor_icon() -> String {
    let user_home = env::var("HOME").unwrap();
    let icon_path = format!("{}/.local/share/icons/keylight-toggle.png", &user_home);
    return icon_path.to_string();
}

#[async_trait]
pub trait Repository {
    async fn list(&self) -> Result<Vec<Keylight>>;
    async fn switch(&self, keylights: Vec<Keylight>, on: bool) -> Result<()>;
}
