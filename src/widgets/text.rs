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

#[derive(Default)]
pub struct TextSettings {
    pub data: WidgetData,
    pub text: String,
    pub background: Option<Color>,
    pub color: Color,
    pub size: f32,
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

    pub fn change_text(&mut self, text: &str) {
        self.layout.clear();
        if let Some(ref mut env) = self.env {
            self.layout.append(
                &env.fonts.fonts(),
                &TextStyle::new(text, self.settings.size, 0),
            );
        }
    }
}

impl Widget for Text {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.env = Some(env);

        let env = self.env.as_mut().unwrap();
        self.layout.append(
            &env.fonts.fonts(),
            &TextStyle::new(&self.settings.text, self.settings.size, 0),
        );

        self.settings.data.height = self.layout.height() as usize;
        self.settings.data.width = 0;
        if let Some(lines) = self.layout.lines() {
            for line in lines {
                let glyph = self.layout.glyphs()[line.glyph_end];
                let width = glyph.width + glyph.x.ceil() as usize;

                self.settings.data.width = usize::max(self.settings.data.width, width);
            }
        }

        Ok(())
    }

    fn draw(&self, drawer: &mut Drawer) -> Result<()> {
        let env = self.env.as_ref().unwrap();
        let fonts = env.fonts.fonts();
        let font = &fonts[0];
        let data = &self.settings.data;

        if let Some(color) = self.settings.background {
            for x in 0..data.width {
                for y in 0..data.height {
                    drawer.draw_pixel(data, (x, y), color);
                }
            }
        }

        for glyph in self.layout.glyphs() {
            drawer.draw_glyph(data, glyph, font, self.settings.color);
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
                &env.fonts.fonts(),
                &TextStyle::new(&settings.text, settings.size, 0),
            );

            settings.data.height = layout.height() as usize;
        }

        Ok(Text {
            layout,

            settings,
            env,
        })
    }
}
