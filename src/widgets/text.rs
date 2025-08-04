use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use anyhow::Result;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

use serde::Deserialize;

use thiserror::Error;

use crate::{
    root::Environment,
    util::{fonts, Color},
    widgets::Widget,
};

use super::{Style, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled};

/// Settings of a [Text] widget
#[derive(Deserialize, Debug, Clone, Default)]
pub struct TextSettings {
    #[serde(default, flatten)]
    pub default_data: WidgetData,

    /// Default text displayed by the widget
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub font_color: Color,

    /// Default font size
    #[serde(default)]
    pub size: f32,

    /// Id of font in vector of fonts for current [crate::util::fonts::FontsMap]
    #[serde(default)]
    pub fontid: usize,

    #[serde(default)]
    pub style: Style,
}

#[derive(Debug, Error)]
pub enum TextError {}

/// Basic widget used for drawing text to a screen
pub struct Text {
    layout: Layout,

    settings: TextSettings,
    data: RefCell<WidgetData>,
    env: Option<Rc<Environment>>,

    is_ready: RefCell<bool>,
}

impl Text {
    /// Text is not cached as a string and gets consturcted every time. Often usage of the function might be pricy.
    pub fn get_text(&self) -> String {
        let mut text = String::new();

        for glyph in self.layout.glyphs() {
            text.push(glyph.parent);
        }

        text
    }

    pub fn change_text(&mut self, text: &str) {
        self.layout.clear();
        if let Some(ref mut _env) = self.env {
            self.layout.append(
                &fonts::fonts_vec(),
                &TextStyle::new(text, self.settings.size, self.settings.fontid),
            );
        }

        self.update_width();
        self.data.borrow_mut().height = self.layout.height() as usize;
    }

    fn update_width(&self) {
        let mut data = self.data.borrow_mut();
        data.width = 0;
        if let Some(lines) = self.layout.lines() {
            for line in lines {
                let glyph = self.layout.glyphs()[line.glyph_end];
                let width = glyph.width + glyph.x.ceil() as usize;

                data.width = usize::max(data.width, width);
            }
        }
    }
}

impl Widget for Text {
    fn name(&self) -> WidgetList {
        WidgetList::Text
    }

    fn as_styled(&self) -> Option<&dyn WidgetStyled> {
        Some(self)
    }

    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError> {
        self.env = Some(env);

        let _env = self.env.as_mut().unwrap();
        self.layout.append(
            &fonts::fonts_vec(),
            &TextStyle::new(
                &self.settings.text,
                self.settings.size,
                self.settings.fontid,
            ),
        );

        Ok(())
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.env.clone()
    }

    fn init(&self) -> Result<(), WidgetError> {
        self.update_width();
        self.data.borrow_mut().height = self.layout.height() as usize;

        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        self.update_width();
        self.apply_style()?;

        *self.is_ready.borrow_mut() = true;
        self.data.borrow_mut().height = self.layout.height() as usize;
        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Text));
        }

        if !*self.is_ready.borrow() {
            self.prepare()?;
        }
        *self.is_ready.borrow_mut() = false;

        self.draw_style()?;

        let font = &fonts::fonts_vec()[self.settings.fontid];
        let data = &self.data.borrow_mut();
        let mut drawer = self.env.as_ref().unwrap().drawer.borrow_mut();

        for glyph in self.layout.glyphs() {
            drawer.draw_glyph(data, glyph, font, self.settings.font_color);
        }

        Ok(())
    }

    fn data(&self) -> Ref<'_, WidgetData> {
        self.data.borrow()
    }

    fn data_mut(&self) -> RefMut<'_, WidgetData> {
        self.data.borrow_mut()
    }
}

impl WidgetNew for Text {
    type Settings = TextSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

        layout.reset(&LayoutSettings {
            max_width: match settings.default_data.width {
                0 => None,
                width => Some(width as f32),
            },
            ..LayoutSettings::default()
        });

        let mut text = Text {
            layout,

            data: RefCell::new(settings.default_data),
            settings,
            env: None,

            is_ready: RefCell::new(false),
        };

        if let Some(e) = env {
            text.bind(e)?;
        }
        Ok(text)
    }
}

impl WidgetStyled for Text {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}
