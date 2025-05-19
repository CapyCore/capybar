use std::rc::Rc;

use chrono::Local;
use fontdue::Font;

use crate::{widget::Label, widget::Widget};

pub struct Clock {
    label: Label,
}

impl Clock {
    pub fn new(font: &Rc<Vec<Font>>, size: f32) -> Self {
        Clock {
            label: Label::new(&Local::now().format("%H:%M:%S").to_string(), font, size),
        }
    }

    pub fn update(&mut self) -> &Self {
        let text = self.label.text();
        text.clear();
        text.push_str(&Local::now().format("%H:%M:%S").to_string());

        self
    }
}

impl Widget for Clock {
    fn draw(
        &mut self,
        canvas: &mut [u8],
        global_offset_x: usize,
        global_offset_y: usize,
        width: usize,
    ) {
        self.update();
        self.label
            .draw(canvas, global_offset_x, global_offset_y, width);
    }
}
