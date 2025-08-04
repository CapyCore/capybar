use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use anyhow::Result;
use chrono::Local;
use serde::Deserialize;

use crate::{
    root::Environment,
    util::Color,
    widgets::{text::Text, Widget},
};

use super::{
    text::TextSettings, Margin, Style, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled,
};

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

    #[serde(default, flatten)]
    pub style: Style,
}

impl Default for ClockSettings {
    fn default() -> Self {
        Self {
            size: 25.0,
            format: default_format(),

            font_color: Color::BLACK,

            default_data: WidgetData::default(),

            style: Style::default(),
        }
    }
}

/// Widget displaying current time. Supports C's strftime formating.
pub struct Clock {
    text: RefCell<Text>,
    settings: ClockSettings,

    data: RefCell<WidgetData>,
    is_ready: RefCell<bool>,
}

impl Clock {
    /// Force update current time  
    pub fn update(&self) -> &Self {
        let mut text = self.text.borrow_mut();
        text.change_text(&Local::now().format(&self.settings.format).to_string());
        text.data_mut().position = self.data.borrow_mut().position;

        self
    }
}

impl Widget for Clock {
    fn name(&self) -> WidgetList {
        WidgetList::Clock
    }

    fn as_styled(&self) -> Option<&dyn WidgetStyled> {
        Some(self)
    }

    fn data(&self) -> Ref<'_, WidgetData> {
        self.data.borrow()
    }

    fn data_mut(&self) -> RefMut<'_, WidgetData> {
        self.data.borrow_mut()
    }

    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError> {
        self.text.borrow_mut().bind(env)
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.text.borrow_mut().env()
    }

    fn init(&self) -> Result<(), WidgetError> {
        let text = self.text.borrow_mut();

        text.init()?;

        let text_data = text.data_mut();
        let mut data = self.data.borrow_mut();

        data.width += text_data.width;
        data.height += text_data.height;

        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        {
            let text = self.text.borrow();
            text.prepare()?;
            let mut it_data = text.data_mut();
            let mut self_data = self.data.borrow_mut();
            it_data.position = self_data.position;
            self_data.width = it_data.width;
            self_data.height = it_data.height;
        }

        self.apply_style()?;

        *self.is_ready.borrow_mut() = true;
        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Clock));
        }
        self.draw_style()?;
        self.update();
        self.text.borrow_mut().draw()
    }
}

impl WidgetNew for Clock {
    type Settings = ClockSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
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

                style: Style {
                    margin: Margin {
                        left: 2,
                        right: 2,
                        up: 0,
                        down: 0,
                    },
                    ..Style::default()
                },

                ..TextSettings::default()
            },
        )?);
        Ok(Clock {
            text,
            data: RefCell::new(settings.default_data),
            settings,
            is_ready: RefCell::new(false),
        })
    }
}

impl WidgetStyled for Clock {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}
