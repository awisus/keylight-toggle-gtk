use crate::keylight::Keylight;

use std::{cell::RefCell, env, process::Command, rc::Rc, sync::Arc};

use async_trait::async_trait;
use gtk::{Menu, MenuItem, SeparatorMenuItem, prelude::*};
use libappindicator::{AppIndicator, AppIndicatorStatus};
use anyhow::Result;

pub struct KeylightUI {
    repository: Arc<dyn Repository>,

    on: Rc<RefCell<Option<bool>>>,
    lights: Rc<RefCell<Vec<Keylight>>>,

    indicator: Rc<RefCell<AppIndicator>>,
    menu: Menu,
    toggle_button: Rc<RefCell<MenuItem>>,
    restart_button: Rc<RefCell<MenuItem>>,
    exit_button: MenuItem,
}

impl KeylightUI {
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        let mut menu = Menu::new();
        let toggle_button = MenuItem::with_label("Suche...");
        toggle_button.set_sensitive(false);
        menu.append(&toggle_button);
        menu.append(&SeparatorMenuItem::new());
        let restart_button = MenuItem::with_label("Neu starten");
        menu.append(&restart_button);
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
            restart_button: Rc::new(RefCell::new(restart_button)),
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
        let repo = self.repository.clone();
        let on = self.on.clone();
        let lights = self.lights.clone();
        let toggle_button = self.toggle_button.clone();
        let restart_button = self.restart_button.clone();

        self.toggle_button.borrow().connect_activate(move |_| {
            let repo = repo.clone();
            let on = on.clone();
            let lights = lights.clone();
            let toggle_button = toggle_button.clone();

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

        self.restart_button.borrow().connect_activate(move |_| {
            restart_button.borrow_mut().set_sensitive(false);

            if let Ok(exe) = env::current_exe() {
                let _ = Command::new(exe).spawn();
            }

            gtk::main_quit();
        });

        self.exit_button.connect_activate(|_| {
            gtk::main_quit();
        });
    }

    fn discover(&self) {
        let repo = self.repository.clone();
        let on = self.on.clone();
        let lights = self.lights.clone();
        let toggle_button = self.toggle_button.clone();

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
