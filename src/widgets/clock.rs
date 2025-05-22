use std::rc::Rc;

use chrono::Local;
use fontdue::Font;

use crate::{
    util::Drawer,
    widgets::{Text, Widget},
};

use super::WidgetData;

pub struct Clock {
    label: Text,
}

impl Clock {
    pub fn new(fonts: &Rc<Vec<Font>>, size: f32) -> Self {
        Clock {
            label: Text::new(
                Local::now().format("%H:%M:%S").to_string(),
                &mut Rc::clone(fonts),
                size,
                WidgetData {
                    width: (size * 5.0) as usize,
                    ..WidgetData::new()
                },
            ),
        }
    }

    pub fn update(&mut self) -> &Self {
        self.label
            .change_text(&Local::now().format("%H:%M:%S").to_string());

        self
    }
}

impl Widget for Clock {
    fn draw(&mut self, drawer: &mut Drawer) {
        self.update();
        self.label.draw(drawer);
    }

    fn data(&mut self) -> &mut super::WidgetData {
        self.label.data()
    }
}
