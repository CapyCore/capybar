use serde::Deserialize;

use crate::widgets::{containers::bar::BarSettings, WidgetsSettingsList};

#[derive(Default, Deserialize, Debug)]
pub struct Bar {
    #[serde(default)]
    pub settings: BarSettings,
    #[serde(default)]
    pub left: Vec<WidgetsSettingsList>,
    #[serde(default)]
    pub center: Vec<WidgetsSettingsList>,
    #[serde(default)]
    pub right: Vec<WidgetsSettingsList>,
}

impl Bar {
    pub const fn default() -> Self {
        Self {
            settings: BarSettings::default(),
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }
}
