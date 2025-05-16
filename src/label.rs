use fontdue::Metrics;

use crate::widget::Widget;

pub struct Label {
    text: String,
    font: fontdue::Font,
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
    pub fn new(text: &str, font: &fontdue::Font, size: f32) -> Self {
        Label {
            text: text.to_string(),
            font: font.clone(),
            size,

            data: DrawingData {
                carriage: 0,
                g_off_x: 0,
                g_off_y: 0,
                width: 0,
                ascent: 0.0,
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

        self.data.update(
            global_offset_x,
            global_offset_y,
            width,
            self.font.horizontal_line_metrics(self.size).unwrap().ascent,
        );

        for character in self.text.chars() {
            let (metrics, bitmap) = self.font.rasterize_subpixel(character, self.size);

            match character {
                ' ' => self.data.carriage += (metrics.advance_width.round() as usize) * 4,
                _ => draw_char(&mut self.data, &metrics, &bitmap, canvas),
            }
        }
    }
}

fn draw_char(data: &mut DrawingData, metrics: &Metrics, bitmap: &[u8], canvas: &mut [u8]) {
    let offset_y = (data.ascent - metrics.bounds.height - metrics.bounds.ymin).round() as usize
        * data.width
        * 4;
    for y in 0..metrics.height {
        for x in 0..metrics.width {
            let a: u8 = 0xFF;
            let r: u8 = bitmap[x * 3 + y * metrics.width * 3];
            let g: u8 = bitmap[x * 3 + y * metrics.width * 3 + 1];
            let b: u8 = bitmap[x * 3 + y * metrics.width * 3 + 2];

            let global_offset =
                (data.carriage + (data.g_off_y + y) * 4 * data.width + (x + data.g_off_x) * 4)
                    + offset_y;
            canvas[global_offset] = b;
            canvas[global_offset + 1] = g;
            canvas[global_offset + 2] = r;
            canvas[global_offset + 3] = a;
        }
    }

    data.carriage += (metrics.width) * 4;
}
