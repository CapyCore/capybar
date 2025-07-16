//! Current module describes all of capybars included processes as well as their common behaviour.
//!
//! Process can be treated as a backend component.
//! To communicate with frontend you can use [Signal](crate::util::signals::Signal)

pub mod clients;

use std::rc::Rc;

use serde::Deserialize;
use thiserror::Error;

use crate::root::Environment;

fn default_update_rate() -> i64 {
    1000
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ProcessSettings {
    #[serde(default = "default_update_rate")]
    pub update_rate: i64,
}

/// A **data structure** that can be used as a widget inside a capybar.
pub trait Process {
    /// Bind a widget to a new environment.
    fn bind(&mut self, env: Rc<Environment>) -> Result<(), ProcessError>;

    /// Prepare `Process` for a first run
    fn init(&self) -> Result<(), ProcessError>;

    /// Run the process
    fn run(&self) -> Result<(), ProcessError>;
}

/// A `Process` that can be unifiedly created.
///
/// Implementing this trait allows creating `Process` and binding the environment without
/// intermidiate steps. Simplifies process creation inside of scripts.
pub trait ProcessNew {
    type Settings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, ProcessError>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum ProcessError {
    /// Argument is a name of a widget
    #[error("Trying to run a procces \"{0}\" not bound to any environment")]
    RunWithNoEnv(String),

    #[error("Custom error occured in widget \"{0}\": \n \"{1}\"")]
    Custom(String, anyhow::Error),
}
