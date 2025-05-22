use std::rc::Rc;

use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font,
};

use crate::{util::Drawer, widgets::Widget};

use super::WidgetData;

pub struct Text {
    layout: Layout,
    fonts: Rc<Vec<Font>>,
    size: f32,

    data: WidgetData,
}

impl Text {
    pub fn new(text: String, fonts: &mut Rc<Vec<Font>>, size: f32, mut data: WidgetData) -> Self {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

        layout.reset(&LayoutSettings {
            max_width: match data.width {
                0 => None,
                width => Some(width as f32),
            },
            ..LayoutSettings::default()
        });

        layout.append(&Rc::make_mut(fonts), &TextStyle::new(&text, size, 0));

        data.height = layout.height().clone() as usize;

        Text {
            layout,
            fonts: Rc::clone(fonts),
            size,

            data,
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
            &TextStyle::new(&text, self.size, 0),
        );
    }
}

impl Widget for Text {
    fn draw(&mut self, drawer: &mut Drawer) {
        let font = &Rc::make_mut(&mut self.fonts)[0];

        for glyph in self.layout.glyphs() {
            drawer.draw_glyph(&self.data, glyph, font);
        }
    }

    fn data(&mut self) -> &mut WidgetData {
        &mut self.data
    }
}
