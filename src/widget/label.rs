use std::rc::Rc;

use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, Metrics,
};

use crate::{util::Color, widget::Widget};

pub struct Label {
    text: String,
    fonts: Rc<Vec<Font>>,
    size: f32,

    data: DrawingData,
}

struct DrawingData {
    pub carriage: usize,
    pub g_off_x: usize,
    pub g_off_y: usize,
    pub width: usize,
    pub ascent: f32,
}

impl Default for DrawingData {
    fn default() -> DrawingData {
        DrawingData {
            carriage: 0,
            g_off_x: 0,
            g_off_y: 0,
            width: 0,
            ascent: 0.0,
        }
    }
}

impl DrawingData {
    pub fn update(&mut self, g_off_x: usize, g_off_y: usize, width: usize, ascent: f32) {
        self.carriage = 0;
        self.g_off_x = g_off_x;
        self.g_off_y = g_off_y;
        self.width = width;
        self.ascent = ascent;
    }
}

impl Label {
    pub fn new(text: &str, font: &Rc<Vec<Font>>, size: f32) -> Self {
        Label {
            text: text.to_string(),
            fonts: Rc::clone(font),
            size,

            data: DrawingData {
                ..Default::default()
            },
        }
    }

    pub fn text(&mut self) -> &mut String {
        &mut self.text
    }
}

impl Widget for Label {
    fn draw(
        &mut self,
        canvas: &mut [u8],
        global_offset_x: usize,
        global_offset_y: usize,
        width: usize,
    ) {
        if self.text.len() == 0 {
            return;
        }

        let mut layout = Layout::new(CoordinateSystem::PositiveYUp);

        layout.reset(&LayoutSettings {
            max_width: Some(10.0),
            ..LayoutSettings::default()
        });

        let text = self.text().clone();
        let fonts = Rc::make_mut(&mut self.fonts);

        layout.append(&fonts, &TextStyle::new(&text, self.size, 0));

        self.data.update(
            global_offset_x,
            global_offset_y,
            width,
            fonts[0].horizontal_line_metrics(self.size).unwrap().ascent,
        );

        //TODO rewrite with layouts
        for glyph in layout.glyphs() {
            let (metrics, bitmap) =
                fonts[0].rasterize_indexed_subpixel(glyph.key.glyph_index, glyph.key.px);
            if glyph.char_data.is_whitespace() {
                self.data.carriage += metrics.advance_width.round() as usize;
                continue;
            }

            draw_char(&mut self.data, &metrics, &bitmap, canvas);
        }
    }
}

fn draw_char(data: &mut DrawingData, metrics: &Metrics, bitmap: &[u8], canvas: &mut [u8]) {
    let offset_y =
        (data.ascent - metrics.bounds.height - metrics.bounds.ymin).round() as usize * data.width;
    for y in 0..metrics.height {
        for x in 0..metrics.width {
            let a: u8 = 0xFF;
            let r: u8 = bitmap[x * 3 + y * metrics.width * 3];
            let g: u8 = bitmap[x * 3 + y * metrics.width * 3 + 1];
            let b: u8 = bitmap[x * 3 + y * metrics.width * 3 + 2];

            let color = Color::from_rgba(r, g, b, a);

            let global_offset =
                data.carriage + (data.g_off_y + y) * data.width + (x + data.g_off_x) + offset_y;

            let chunk = canvas.chunks_exact_mut(4).nth(global_offset).unwrap();
            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = color.to_rgba();
        }
    }

    data.carriage += metrics.width;
}
