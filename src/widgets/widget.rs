use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    root::Environment,
    util::{Color, Drawer},
};

use super::{
    battery::BatterySettings,
    clock::ClockSettings,
    cpu::CPUSettings,
    text::TextSettings,
};

pub trait Widget {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()>;
    fn draw(&self, drawer: &mut Drawer) -> Result<()>;
    fn init(&self) -> Result<()>;
    fn data(&self) -> &RefCell<WidgetData>;
}

pub trait WidgetNew: Widget {
    type Settings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum WidgetError {
    #[error("Invalid widget bounds")]
    InvalidBounds,
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct WidgetData {
    #[serde(default)]
    pub position: (usize, usize),
    #[serde(default)]
    pub width: usize,
    #[serde(default)]
    pub height: usize,
    #[serde(default)]
    pub margin: (usize, usize, usize, usize),
}

impl WidgetData {
    pub const fn default() -> Self {
        Self {
            position: (0, 0),
            width: 0,
            height: 0,
            margin: (0, 0, 0, 0),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct Style {
    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
}

impl Style {
    pub const fn default() -> Self {
        Self {
            background: None,
            border: None,
        }
    }
}

pub trait WidgetStyled: Widget {
    fn style(&self) -> &Style;

    fn style_mut(&mut self) -> &mut Style;

    fn apply_style(&self) -> Result<()> {
        let mut data = self.data().borrow_mut();
        let style = self.style();

        let border = match style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        data.height += border.0 * 2;

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "widget", content = "settings", rename_all = "snake_case")]
pub enum WidgetsList {
    Text(TextSettings),
    Clock(ClockSettings),
    Battery(BatterySettings),
    #[serde(rename = "cpu")]
    CPU(CPUSettings),
}
