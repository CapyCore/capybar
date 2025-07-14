use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use chrono::{DateTime, Duration, Local};
use hyprland::{data::Devices, shared::HyprData};

use crate::{
    processes::{clients::KeyboardTrait, Process, ProcessNew, ProcessSettings},
    root::Environment,
    util::signals::Signal,
};

/// Process that tracks current keyboard layout
pub struct Keyboard {
    settings: ProcessSettings,

    last_layout: RefCell<String>,
    last_update: RefCell<DateTime<Local>>,

    env: Option<Rc<Environment>>,
}

impl Process for Keyboard {
    fn bind(&mut self, env: std::rc::Rc<crate::root::Environment>) -> anyhow::Result<()> {
        self.env = Some(Rc::clone(&env));
        Ok(())
    }

    fn init(&self) -> anyhow::Result<()> {
        if self.env.is_none() {
            return Err(anyhow!("No Environment is bound"));
        }

        let mut signals = self.env.as_ref().unwrap().signals.borrow_mut();
        if !signals.contains_key("keyboard") {
            signals.insert("keyboard".to_string(), Signal::new());
        }

        let keyboards = Devices::get()?.keyboards;
        if keyboards.is_empty() {
            return Err(anyhow!("No Keyboard connected"));
        }

        let mut last_layout = self.last_layout.borrow_mut();
        for keyboard in keyboards {
            if keyboard.main {
                *last_layout = keyboard.layout;
            }
        }

        signals["keyboard"].emit(&last_layout.clone());

        Ok(())
    }

    fn run(&self) -> anyhow::Result<()> {
        if self.env.is_none() {
            return Err(anyhow!("No Environment is bound"));
        }

        let mut last_update = self.last_update.borrow_mut();
        if Local::now() - *last_update < Duration::milliseconds(self.settings.update_rate) {
            return Ok(());
        }
        *last_update = Local::now();

        let keyboards = Devices::get()?.keyboards;

        if keyboards.is_empty() {
            return Err(anyhow!("No Keyboard connected"));
        }

        let signals = self.env.as_ref().unwrap().signals.borrow_mut();
        let mut last_layout = self.last_layout.borrow_mut();
        for keyboard in keyboards {
            if keyboard.main && *last_layout != keyboard.active_keymap {
                *last_layout = keyboard.active_keymap;
                signals["keyboard"].emit(&last_layout.clone());
            }
        }

        Ok(())
    }
}

impl ProcessNew for Keyboard {
    type Settings = ProcessSettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let keyboards = Devices::get()?.keyboards;
        if keyboards.is_empty() {
            return Err(anyhow!("No Keyboard connected"));
        }

        for keyboard in keyboards {
            if keyboard.main {
                return Ok(Keyboard {
                    settings,
                    last_update: RefCell::new(DateTime::default()),
                    last_layout: RefCell::new(String::new()),
                    env,
                });
            }
        }

        return Err(anyhow!("No main keyboard found"));
    }
}

impl KeyboardTrait for Keyboard {}
