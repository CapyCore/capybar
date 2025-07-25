use std::{cell::RefCell, rc::Rc};

use anyhow::anyhow;
use chrono::{DateTime, Duration, Local};
use hyprland::{data::Devices, shared::HyprData};

use crate::{
    processes::{clients::KeyboardTrait, Process, ProcessError, ProcessNew, ProcessSettings},
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

impl Keyboard {
    fn get_main_keyboard() -> Result<hyprland::data::Keyboard, ProcessError> {
        let devices = Devices::get();
        if let Err(err) = devices {
            return Err(ProcessError::Custom("Keyboard".to_string(), err.into()));
        }

        let keyboards = devices.unwrap().keyboards;

        if keyboards.is_empty() {
            return Err(ProcessError::Custom(
                "Keyboard".to_string(),
                anyhow!("No Keyboard connected"),
            ));
        }

        for keyboard in keyboards {
            if keyboard.main {
                return Ok(keyboard);
            }
        }

        Err(ProcessError::Custom(
            "Keyboard".to_string(),
            anyhow!("No main keyboard found"),
        ))
    }
}

impl Process for Keyboard {
    fn bind(&mut self, env: std::rc::Rc<crate::root::Environment>) -> Result<(), ProcessError> {
        self.env = Some(Rc::clone(&env));
        Ok(())
    }

    fn init(&self) -> Result<(), ProcessError> {
        if self.env.is_none() {
            return Err(ProcessError::RunWithNoEnv("Keyboard".to_string()));
        }

        let mut signals = self.env.as_ref().unwrap().signals.borrow_mut();
        if !signals.contains_key("keyboard") {
            signals.insert("keyboard".to_string(), Signal::new());
        }

        *self.last_layout.borrow_mut() = Keyboard::get_main_keyboard()?.active_keymap;
        signals["keyboard"].emit(&self.last_layout.clone());

        Ok(())
    }

    fn run(&self) -> Result<(), ProcessError> {
        if self.env.is_none() {
            return Err(ProcessError::RunWithNoEnv("Keyboard".to_string()));
        }

        let mut last_update = self.last_update.borrow_mut();
        if Local::now() - *last_update < Duration::milliseconds(self.settings.update_rate) {
            return Ok(());
        }
        *last_update = Local::now();

        let signals = self.env.as_ref().unwrap().signals.borrow_mut();
        let mut last_layout = self.last_layout.borrow_mut();
        let current_layout = Keyboard::get_main_keyboard()?.active_keymap;
        if *last_layout != current_layout {
            *last_layout = current_layout;
            signals["keyboard"].emit(&last_layout.clone());
        }

        Ok(())
    }
}

impl ProcessNew for Keyboard {
    type Settings = ProcessSettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> Result<Self, ProcessError>
    where
        Self: Sized,
    {
        Ok(Keyboard {
            settings,
            last_update: RefCell::new(DateTime::default()),
            last_layout: RefCell::new(String::new()),
            env,
        })
    }
}

impl KeyboardTrait for Keyboard {}
