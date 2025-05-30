use std::{cell::RefCell, rc::Rc};

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
    label: RefCell<Text>,
    settings: ClockSettings,
}

impl Clock {
    pub fn update(&self) -> &Self {
        self.label
            .borrow_mut()
            .change_text(&Local::now().format(&self.settings.format).to_string());

        self
    }
}

impl Widget for Clock {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.label.borrow_mut().bind(env)
    }

    fn draw(&self, drawer: &mut Drawer) -> Result<()> {
        self.update();
        self.label.borrow_mut().draw(drawer)
    }

    fn data(&mut self) -> Result<&mut WidgetData> {
        self.label.get_mut().data()
    }
}

impl WidgetNew for Clock {
    type Settings = ClockSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Clock {
            label: RefCell::new(Text::new(
                env,
                TextSettings {
                    text: Local::now().format(&settings.format).to_string(),
                    color: settings.font_color,
                    size: settings.size,

                    data: WidgetData {
                        width: (settings.size * 6.0) as usize,
                        ..WidgetData::default()
                    },

                    ..TextSettings::default()
                },
            )?),
            settings,
        })
    }
}
