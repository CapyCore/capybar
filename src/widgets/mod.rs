pub mod containers;

pub mod battery;
pub mod clock;
pub mod cpu;
pub mod icon_text;
pub mod keyboard;
pub mod text;

use std::{
    cell::{Ref, RefMut},
    fmt::Display,
    ops::{Add, AddAssign},
    rc::Rc,
};

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    root::Environment,
    services::{ProcessSettings, ServiceList, ServiceNew},
    util::Color,
};

use {battery::BatterySettings, clock::ClockSettings, cpu::CPUSettings, text::TextSettings};

/// A **data structure** that can be used as a widget inside a capybar.
pub trait Widget {
    /// Get type of a curent widget
    fn name(&self) -> WidgetList;

    /// Bind a widget to a new environment.
    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError>;

    /// Get environment bound to the widget
    fn env(&self) -> Option<Rc<Environment>>;

    /// Prepare current widget for a draw
    fn prepare(&self) -> Result<(), WidgetError> {
        todo!()
    }

    /// Draw an entire widget to a current environment's `Drawer`
    fn draw(&self) -> Result<(), WidgetError>;

    /// Prepare [Widget] for a first draw
    fn init(&self) -> Result<(), WidgetError>;

    /// Return [WidgetData] associated to the widget immutably
    fn data(&self) -> Ref<'_, WidgetData>;

    /// Return [WidgetData] associated to the widget mutably
    fn data_mut(&self) -> RefMut<'_, WidgetData>;

    fn try_data(&self) -> Ref<'_, WidgetData> {
        todo!()
    }

    fn try_data_mut(&self) -> RefMut<'_, WidgetData> {
        todo!()
    }

    /// Runtime check if widget is styled
    /// Override this function with body `Some(self)` if [WidgetStyled] implemented
    fn as_styled(&self) -> Option<&dyn WidgetStyled> {
        None
    }
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
    DrawWithNoEnv(WidgetList),

    /// Argument is a name of a widget
    #[error("Trying to initialise a widget \"{0}\" not bound to any environment")]
    InitWithNoEnv(WidgetList),

    /// Arguments are a name of a widget and a name of coresponding service
    #[error(
        "When initialising widget \"{0}\" no coresponding signal was found.
        Maybe service \"{1}\" was not created?"
    )]
    NoCorespondingSignal(WidgetList, ServiceList),

    #[error(
        "Style initialisation is invalid in widget \"{0}\". Data can not be borrowed mutably."
    )]
    StyleInitDataBorrowed(WidgetList),

    #[error(transparent)]
    Custom(#[from] anyhow::Error),
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct Position(pub usize, pub usize);

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl AddAssign<(usize, usize)> for Position {
    fn add_assign(&mut self, rhs: (usize, usize)) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Add<(usize, usize)> for Position {
    type Output = Position;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

/// Global common data used by `Widget` data structure.
#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct WidgetData {
    /// Offset of the widget in a global scope. Usually controlled by parent.
    #[serde(default)]
    pub position: Position,

    /// Widgth of the widget should be controlled by the widget itself
    #[serde(default)]
    pub width: usize,

    /// Height of the widget should be controlled by the widget itself
    #[serde(default)]
    pub height: usize,
}

impl WidgetData {
    pub const fn default() -> Self {
        Self {
            position: Position(0, 0),
            width: 0,
            height: 0,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct Margin {
    pub left: usize,
    pub right: usize,
    pub up: usize,
    pub down: usize,
}

impl Margin {
    pub const fn default() -> Self {
        Self {
            left: 0,
            right: 0,
            up: 0,
            down: 0,
        }
    }
}

/// Common style used by `Widget`
#[derive(Default, Debug, Clone, Copy, Deserialize)]
pub struct Style {
    pub background: Option<Color>,

    /// Border of a pixel (border pixel width, color)
    pub border: Option<(usize, Color)>,

    /// Margin of a widget (Left, Right, Up, Down)
    #[serde(default)]
    pub margin: Margin,
}

impl Style {
    pub const fn default() -> Self {
        Self {
            background: None,
            border: None,
            margin: Margin::default(),
        }
    }
}

/// [Widget] that supports common styling. Already provides helper functions fot initialising and drawing styled widget.
pub trait WidgetStyled: Widget {
    fn style(&self) -> &Style;

    /// Helper function that you should call before alignment, since it changes dimensions
    ///
    /// <div class="warning">
    /// <b> Before using default function make sure you understand and follow these points:</b> <br>
    /// <ul>
    /// <li> Borrows [WidgetData] via calling [Widget::data_mut()] then
    /// adds border and margins to width and height; <br> </li>
    /// <li> Borrows [Style] immutably; <br> </li>
    /// <li> Should be called once after every width or height are overwritten. Otherwise width, height and position will be innacurate. <br> </li>
    /// </ul>
    /// </div>
    fn apply_style(&self) -> Result<(), WidgetError> {
        let mut data = self.data_mut();
        let style = self.style();

        let border = match style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        data.height += border.0 * 2;

        data.width += style.margin.left + style.margin.right;
        data.height += style.margin.up + style.margin.down;

        Ok(())
    }

    /// Helper function that you should call in the begining of draw function.
    /// <div class="warning">
    /// <b> Before using default function make sure you understand and follow these points:</b> <br>
    /// <ul>
    /// <li> Borrows [WidgetData] via calling [Widget::data_mut()] then
    /// adds margins to position; <br> </li>
    /// <li> Borrows [Style] immutably; <br> </li>
    /// <li> Draws the background and border, therefore should be called every draw before the main
    /// logic<br> </li>
    /// </ul>
    /// </div>
    fn draw_style(&self) -> Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(self.name()));
        }

        let env = self.env().unwrap();
        let style = self.style();
        let border = style.border.unwrap_or((0, Color::NONE));
        let mut data = self.data_mut();

        data.position.0 += style.margin.left;
        data.position.1 += style.margin.up;

        let mut drawer = env.as_ref().drawer.borrow_mut();
        if let Some(color) = style.background {
            for x in border.0..data.width - border.0 {
                for y in border.0..data.height - border.0 {
                    drawer.draw_pixel(&data, (x, y), color);
                }
            }
        }

        if border.1 == Color::NONE {
            return Ok(());
        }

        for x in 0..border.0 {
            for y in 0..data.height {
                drawer.draw_pixel(&data, (x, y), border.1);
                drawer.draw_pixel(&data, (data.width - 1 - x, y), border.1);
            }
        }

        for x in 0..data.width {
            for y in 0..border.0 {
                drawer.draw_pixel(&data, (x, y), border.1);
                drawer.draw_pixel(&data, (x, data.height - 1 - y), border.1);
            }
        }

        Ok(())
    }
}

/// All available widgets in capybar
#[derive(Debug, Clone)]
pub enum WidgetList {
    Text,
    IconText,
    Clock,
    Battery,
    CPU,
    Keyboard,

    Row,
    Bar,

    Custom(String),
}

impl Display for WidgetList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "Text"),
            Self::IconText => write!(f, "Text"),
            Self::Clock => write!(f, "Clock"),
            Self::Battery => write!(f, "Battery"),
            Self::CPU => write!(f, "Cpu"),
            Self::Keyboard => write!(f, "Keyboard"),

            Self::Row => write!(f, "Row"),
            Self::Bar => write!(f, "Bar"),

            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Enum of [Widget]s with their settings.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "widget", content = "settings", rename_all = "snake_case")]
pub enum WidgetsSettingsList {
    Text(TextSettings),
    Clock(ClockSettings),
    Battery(BatterySettings),
    #[serde(rename = "cpu")]
    CPU(CPUSettings),
    Keyboard(keyboard::KeyboardSettings, ProcessSettings),
    Custom(String),
}

impl WidgetsSettingsList {
    pub fn create_in_container(
        &self,
        container: &mut impl containers::ContainerSingle,
    ) -> Result<(), WidgetError> {
        match self {
            WidgetsSettingsList::Text(settings) => {
                container.create_widget(text::Text::new, settings.clone())
            }
            WidgetsSettingsList::Clock(settings) => {
                container.create_widget(clock::Clock::new, settings.clone())
            }
            WidgetsSettingsList::Battery(settings) => {
                container.create_widget(battery::Battery::new, settings.clone())
            }
            WidgetsSettingsList::CPU(settings) => {
                container.create_widget(cpu::CPU::new, settings.clone())
            }
            WidgetsSettingsList::Keyboard(wsettings, psettings) => {
                container.create_service(crate::services::clients::Keyboard::new, *psettings)?;
                container.create_widget(keyboard::Keyboard::new, wsettings.clone())
            }
            WidgetsSettingsList::Custom(_) => {
                todo!()
            }
        }
    }
}
