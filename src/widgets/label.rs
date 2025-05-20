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
            max_width: Some(data.width as f32),
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
            let (_, bitmap) = font.rasterize_indexed_subpixel(glyph.key.glyph_index, glyph.key.px);
            if glyph.char_data.is_whitespace() {
                continue;
            }

            for x in 0..glyph.width {
                for y in 0..glyph.height {
                    let color = Color::from_rgba(
                        bitmap[x * 3 + y * glyph.width * 3],
                        bitmap[x * 3 + y * glyph.width * 3 + 1],
                        bitmap[x * 3 + y * glyph.width * 3 + 2],
                        0xFF,
                    );

                    drawer.draw_pixel(
                        &self.data,
                        (x + glyph.x as usize, y + glyph.y as usize),
                        color,
                    );
                }
            }
        }
    }

    fn data(&mut self) -> &mut WidgetData {
        &mut self.data
    }
}
