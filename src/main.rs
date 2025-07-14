use std::{fmt::Display, path::PathBuf};

use anyhow::Result;
use capybar::{config::Config, root::Root};
use clap::{Args, Parser, ValueEnum};
use std::env::var;
use thiserror::Error;
use wayland_client::{globals::registry_queue_init, Connection};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    args: Arguments,
}

#[derive(Debug, Args)]
struct Arguments {
    /// What config type to use
    #[arg(long, value_enum, default_value_t = ConfigTypes::Toml, value_name = "TYPE")]
    cfg_type: ConfigTypes,

    #[arg(long, value_name = "FILE")]
    /// Directory where the config is located
    cfg_path: Option<PathBuf>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ConfigTypes {
    Toml,
}

impl Display for ConfigTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigTypes::Toml => write!(f, "toml"),
        }
    }
}

#[derive(Debug, Error)]
enum Errors {
    #[error(
        "Configuration file does not exist! 
        Make sure you are passing `--cfg_type <TYPE>` with correct type if it is not TOML.
        Make sure you provide '--cfg_path <PATH>' with your config file or \
        place it config at `~/.config/capybar/config.<TYPE>"
    )]
    ConfigNotExist,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut cfg_path;
    match cli.args.cfg_path {
        None => {
            if let Ok(config_home) = var("XDG_CONFIG_HOME")
                .or_else(|_| var("HOME").map(|home| format!("{home}/.config")))
            {
                cfg_path = config_home.into();
            } else {
                return Err(Errors::ConfigNotExist.into());
            }
        }
        Some(value) => cfg_path = value,
    }

    if cfg_path.is_dir() {
        cfg_path.push("capybar");
        let file_name = "config.".to_string() + &cli.args.cfg_type.to_string();
        cfg_path.push(file_name);
    }

    if !cfg_path.exists() {
        return Err(Errors::ConfigNotExist.into());
    }

    let config = match cli.args.cfg_type {
        ConfigTypes::Toml => Config::parse_toml(cfg_path)?,
    };

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;

    let mut capybar = Root::new(&globals, &mut event_queue)?;
    capybar.apply_config(config)?;

    capybar.init(&mut event_queue)?.run(&mut event_queue)?;

    Ok(())
}
