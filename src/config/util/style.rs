use serde::Deserialize;

use crate::util::Color;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Background {
    pub enable: bool,
    pub color: Color,
}

impl Background {
    pub const fn default() -> Self {
        Self {
            enable: false,
            color: Color::NONE,
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone, Copy)]
#[serde(default)]
pub struct Border {
    enable: bool,
    color: Color,
    size: u32,
}

impl Border {
    pub const fn default() -> Self {
        Self {
            enable: false,
            color: Color::NONE,
            size: 0,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct StyleConfig {}
