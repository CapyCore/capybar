pub mod util;
pub mod widgets;

use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

use util::font::PreloadedFonts;
use widgets::bar::Bar;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub preloaded_fonts: PreloadedFonts,

    pub bar: Bar,
}

impl Config {
    pub const fn default() -> Self {
        Self {
            preloaded_fonts: PreloadedFonts::default(),
            bar: Bar::default(),
        }
    }

    pub fn parse_toml(file: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(file)?;
        let t: Config = toml::from_str(&content)?;

        Ok(t)
    }
}
