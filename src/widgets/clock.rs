use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use chrono::Local;
use serde::Deserialize;

use crate::{
    root::Environment,
    util::{Color, Drawer},
    widgets::{text::Text, Widget},
};

use super::{text::TextSettings, WidgetData, WidgetNew};

fn default_format() -> String {
    "%H:%M".to_string()
}

/// Settings of a [Clock] widget
#[derive(Deserialize, Debug, Clone)]
pub struct ClockSettings {
    /// Default font size
    #[serde(default)]
    pub size: f32,

    /// Default format strftime format
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default)]
    pub font_color: Color,

    #[serde(default)]
    pub default_data: WidgetData,
}

impl Default for ClockSettings {
    fn default() -> Self {
        Self {
            size: 25.0,
            format: default_format(),

            font_color: Color::BLACK,

            default_data: WidgetData::default(),
        }
    }
}

/// Widget displaying current time. Supports C's strftime formating.
pub struct Clock {
    text: RefCell<Text>,
    settings: ClockSettings,

    data: RefCell<WidgetData>,
}

impl Clock {
    /// Force update current time  
    pub fn update(&self) -> &Self {
        let mut text = self.text.borrow_mut();
        text.change_text(&Local::now().format(&self.settings.format).to_string());
        text.data().borrow_mut().position = self.data.borrow_mut().position;

        self
    }
}

impl Widget for Clock {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.text.borrow_mut().bind(env)
    }

    fn init(&self) -> Result<()> {
        let text = self.text.borrow_mut();

        text.init()?;

        let text_data = text.data().borrow_mut();
        let mut data = self.data.borrow_mut();

        data.width = text_data.width;
        data.height = text_data.height;

        Ok(())
    }

    fn draw(&self, drawer: &mut Drawer) -> Result<()> {
        self.update();
        self.text.borrow_mut().draw(drawer)
    }

    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
    }
}

impl WidgetNew for Clock {
    type Settings = ClockSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        let text = RefCell::new(Text::new(
            env,
            TextSettings {
                text: Local::now().format(&settings.format).to_string(),
                font_color: settings.font_color,
                size: settings.size,

                default_data: WidgetData {
                    width: (settings.size * 6.0) as usize,
                    ..WidgetData::default()
                },

                ..TextSettings::default()
            },
        )?);
        Ok(Clock {
            text,
            data: RefCell::new(settings.default_data),
            settings,
        })
    }
}
