//! Current module describes all of capybars included servicees as well as their common behaviour.
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

/// A **data structure** that can be used as a service inside a capybar.
pub trait Service {
    /// Bind a widget to a new environment.
    fn bind(&mut self, env: Rc<Environment>) -> Result<(), ServiceError>;

    /// Prepare [Service] for a first run
    fn init(&self) -> Result<(), ServiceError>;

    /// Run the [Service]
    fn run(&self) -> Result<(), ServiceError>;
}

/// A [Service] that can be unifiedly created.
///
/// Implementing this trait allows creating [Service] and binding the environment without
/// intermidiate steps. Simplifies service creation inside of scripts.
pub trait ServiceNew {
    type Settings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, ServiceError>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum ServiceError {
    /// Argument is a name of a service
    #[error("Trying to run a service \"{0}\" not bound to any environment")]
    RunWithNoEnv(String),

    #[error("Custom error occured in service \"{0}\": \n \"{1}\"")]
    Custom(String, anyhow::Error),
}
