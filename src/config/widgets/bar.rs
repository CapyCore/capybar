use serde::Deserialize;

use crate::widgets::{containers::bar::BarSettings, WidgetsList};

#[derive(Default, Deserialize, Debug)]
pub struct Bar {
    #[serde(default)]
    pub settings: BarSettings,
    #[serde(default)]
    pub left: Vec<WidgetsList>,
    #[serde(default)]
    pub center: Vec<WidgetsList>,
    #[serde(default)]
    pub right: Vec<WidgetsList>,
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
