pub mod containers;

pub mod battery;
pub mod clock;
pub mod cpu;
pub mod keyboard;
pub mod text;

use std::{cell::RefCell, rc::Rc};

use serde::Deserialize;
use thiserror::Error;

use crate::{processes::ProcessSettings, root::Environment, util::Color};

use {battery::BatterySettings, clock::ClockSettings, cpu::CPUSettings, text::TextSettings};

/// A **data structure** that can be used as a widget inside a capybar.
pub trait Widget {
    /// Bind a widget to a new environment.
    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError>;

    /// Draw an entire widget to a current environment's `Drawer`
    fn draw(&self) -> Result<(), WidgetError>;

    /// Prepare `Widget` for a first draw
    fn init(&self) -> Result<(), WidgetError>;

    /// Return `WidgetData` associated to the widget
    fn data(&self) -> &RefCell<WidgetData>;
}

/// A `Widget` that can be unifiedly created.
///
/// Implementing this trait allows creating `Widget` and binding the environment without
/// intermidiate steps. Simplifies widget creation inside of scripts.
pub trait WidgetNew: Widget {
    type Settings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum WidgetError {
    #[error("Invalid widget bounds")]
    InvalidBounds,

    /// Argument is a name of a widget
    #[error("Trying to draw a widget \"{0}\" not bound to any environment")]
    DrawWithNoEnv(String),

    /// Argument is a name of a widget
    #[error("Trying to initialise a widget \"{0}\" not bound to any environment")]
    InitWithNoEnv(String),

    /// Arguments are a name of a widget and a name of coresponding process
    #[error(
        "When initialising widget \"{0}\" no coresponding signal was found.
        Maybe process \"{1}\" was not created?"
    )]
    NoCorespondingSignal(String, String),

    #[error(transparent)]
    Custom(#[from] anyhow::Error),
}

/// Global common data used by `Widget` data structure.
#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct WidgetData {
    /// Offset of the widget in a global scope. Usually controlled by parent.
    #[serde(default)]
    pub position: (usize, usize),

    /// Widgth of the widget should be controlled by the widget itself
    #[serde(default)]
    pub width: usize,

    /// Height of the widget should be controlled by the widget itself
    #[serde(default)]
    pub height: usize,

    /// Size of an empty space around the widget
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

/// Common style used by `Widget`
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

/// `Widget` that supports common styling.
pub trait WidgetStyled: Widget {
    fn style(&self) -> &Style;

    fn style_mut(&mut self) -> &mut Style;

    fn apply_style(&self) -> Result<(), WidgetError> {
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

/// All available widgets in capybar and their settings
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "widget", content = "settings", rename_all = "snake_case")]
pub enum WidgetsList {
    Text(TextSettings),
    Clock(ClockSettings),
    Battery(BatterySettings),
    #[serde(rename = "cpu")]
    CPU(CPUSettings),
    Keyboard(keyboard::KeyboardSettings, ProcessSettings),
}
