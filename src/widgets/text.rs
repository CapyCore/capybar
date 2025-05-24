use std::rc::Rc;

use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font,
};

use crate::{
    util::{Color, Drawer},
    widgets::Widget,
};

use super::WidgetData;

pub struct TextSettings {
    pub background: Option<Color>,
    pub color: Color,
    pub size: f32,
}

impl TextSettings {
    pub fn default() -> Self {
        TextSettings {
            background: None,
            color: Color::BLACK,
            size: 25.0,
        }
    }
}

pub struct Text {
    layout: Layout,
    fonts: Rc<Vec<Font>>,

    data: WidgetData,
    settings: TextSettings,
}

impl Text {
    pub fn new(
        text: String,
        fonts: &mut Rc<Vec<Font>>,
        mut data: WidgetData,
        settings: TextSettings,
    ) -> Self {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

        layout.reset(&LayoutSettings {
            max_width: match data.width {
                0 => None,
                width => Some(width as f32),
            },
            ..LayoutSettings::default()
        });

        layout.append(
            &Rc::make_mut(fonts),
            &TextStyle::new(&text, settings.size, 0),
        );

        data.height = layout.height().clone() as usize;

        Text {
            layout,
            fonts: Rc::clone(fonts),

            data,
            settings,
        }
    }

    pub fn get_text(&self) -> String {
        let mut text = String::new();

        for glyph in self.layout.glyphs() {
            text.push(glyph.parent);
        }

        text
    }

    pub fn change_text(&mut self, text: &String) {
        self.layout.clear();
        self.layout.append(
            &Rc::make_mut(&mut self.fonts),
            &TextStyle::new(&text, self.settings.size, 0),
        );
    }
}

impl Widget for Text {
    fn draw(&mut self, drawer: &mut Drawer) {
        let font = &Rc::make_mut(&mut self.fonts)[0];

        if let Some(color) = self.settings.background {
            for x in 0..self.data.width {
                for y in 0..self.data.height {
                    drawer.draw_pixel(&self.data, (x, y), color);
                }
            }
        }

        for glyph in self.layout.glyphs() {
            drawer.draw_glyph(&self.data, glyph, font, self.settings.color);
        }
    }

    fn data(&mut self) -> &mut WidgetData {
        &mut self.data
    }
}
