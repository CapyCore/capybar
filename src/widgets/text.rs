use std::rc::Rc;

use anyhow::Result;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

use thiserror::Error;

use crate::{
    root::Environment,
    util::{Color, Drawer},
    widgets::Widget,
};

use super::{WidgetData, WidgetNew};

pub struct TextSettings {
    pub data: WidgetData,
    pub text: String,
    pub background: Option<Color>,
    pub color: Color,
    pub size: f32,
}

impl TextSettings {
    pub fn default() -> Self {
        TextSettings {
            data: WidgetData::default(),
            text: String::new(),
            background: None,
            color: Color::BLACK,
            size: 25.0,
        }
    }
}

#[derive(Debug, Error)]
pub enum TextError {}

pub struct Text {
    layout: Layout,

    settings: TextSettings,
    env: Option<Rc<Environment>>,
}

impl Text {
    pub fn get_text(&self) -> String {
        let mut text = String::new();

        for glyph in self.layout.glyphs() {
            text.push(glyph.parent);
        }

        text
    }

    pub fn change_text(&mut self, text: &String) {
        self.layout.clear();
        if let Some(ref mut env) = self.env {
            self.layout.append(
                &mut env.fonts.fonts(),
                &TextStyle::new(&text, self.settings.size, 0),
            );
        }
    }
}

impl Widget for Text {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.env = Some(env);

        let env = &mut self.env.as_mut().unwrap();
        self.layout.append(
            &mut env.fonts.fonts(),
            &TextStyle::new(&self.settings.text, self.settings.size, 0),
        );

        self.settings.data.height = self.layout.height().clone() as usize;

        Ok(())
    }

    fn draw(&mut self, drawer: &mut Drawer) -> Result<()> {
        let env = &mut self.env.as_mut().unwrap();
        let fonts = env.fonts.fonts();
        let font = &fonts[0];
        let data = &self.settings.data;

        if let Some(color) = self.settings.background {
            for x in 0..data.width {
                for y in 0..data.height {
                    drawer.draw_pixel(&data, (x, y), color);
                }
            }
        }

        for glyph in self.layout.glyphs() {
            drawer.draw_glyph(&data, glyph, font, self.settings.color);
        }

        Ok(())
    }

    fn data(&mut self) -> Result<&mut WidgetData> {
        Ok(&mut self.settings.data)
    }
}

impl WidgetNew for Text {
    type Settings = TextSettings;

    fn new(mut env: Option<Rc<Environment>>, mut settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

        layout.reset(&LayoutSettings {
            max_width: match settings.data.width {
                0 => None,
                width => Some(width as f32),
            },
            ..LayoutSettings::default()
        });

        if let Some(ref mut env) = env {
            layout.append(
                &mut env.fonts.fonts(),
                &TextStyle::new(&settings.text, settings.size, 0),
            );

            settings.data.height = layout.height().clone() as usize;
        }

        Ok(Text {
            layout,

            settings,
            env,
        })
    }
}
