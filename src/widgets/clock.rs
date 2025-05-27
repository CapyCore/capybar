use std::rc::Rc;

use anyhow::Result;
use chrono::Local;

use crate::{
    root::Environment,
    util::{Color, Drawer},
    widgets::{text::Text, Widget},
};

use super::{text::TextSettings, WidgetData, WidgetNew};

pub struct ClockSettings {
    pub size: f32,
    pub format: String,

    pub font_color: Color,
}

impl Default for ClockSettings {
    fn default() -> Self {
        Self {
            size: 25.0,
            format: "%H:%M:%S".to_string(),

            font_color: Color::BLACK,
        }
    }
}

pub struct Clock {
    label: Text,
    settings: ClockSettings,
}

impl Clock {
    pub fn update(&mut self) -> &Self {
        self.label
            .change_text(&Local::now().format(&self.settings.format).to_string());

        self
    }
}

impl Widget for Clock {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.label.bind(env)
    }

    fn draw(&mut self, drawer: &mut Drawer) -> Result<()> {
        self.update();
        self.label.draw(drawer)
    }

    fn data(&mut self) -> Result<&mut WidgetData> {
        self.label.data()
    }
}

impl WidgetNew for Clock {
    type Settings = ClockSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Clock {
            label: Text::new(
                env,
                TextSettings {
                    text: Local::now().format(&settings.format).to_string(),
                    color: settings.font_color,

                    data: WidgetData {
                        width: (settings.size * 6.0) as usize,
                        ..WidgetData::default()
                    },

                    ..TextSettings::default()
                },
            )?,
            settings,
        })
    }
}
