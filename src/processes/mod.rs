//! Current module describes all of capybars included processes as well as their common behaviour.
//!
//! Process can be treated as a backend component.
//! To communicate with frontend you can use [Signal](crate::util::signals::Signal)

pub mod clients;

use std::rc::Rc;

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

use crate::root::Environment;

fn default_update_rate() -> i64 {
    return 1000;
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ProcessSettings {
    #[serde(default = "default_update_rate")]
    pub update_rate: i64,
}

/// A **data structure** that can be used as a widget inside a capybar.
pub trait Process {
    /// Bind a widget to a new environment.
    fn bind(&mut self, env: Rc<Environment>) -> Result<()>;

    /// Prepare `Process` for a first run
    fn init(&self) -> Result<()>;

    /// Run the process
    fn run(&self) -> Result<()>;
}

/// A `Process` that can be unifiedly created.
///
/// Implementing this trait allows creating `Process` and binding the environment without
/// intermidiate steps. Simplifies process creation inside of scripts.
pub trait ProcessNew {
    type Settings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum ProcessError {}
