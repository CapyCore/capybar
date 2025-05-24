use std::rc::Rc;

use chrono::Local;
use fontdue::Font;

use crate::{
    util::Drawer,
    widgets::{text::Text, Widget},
};

use super::{text::TextSettings, WidgetData};

pub struct Clock {
    label: Text,
}

impl Clock {
    pub fn new(fonts: &Rc<Vec<Font>>, size: f32) -> Self {
        Clock {
            label: Text::new(
                Local::now().format("%H:%M:%S").to_string(),
                &mut Rc::clone(fonts),
                WidgetData {
                    width: (size * 6.0) as usize,
                    ..WidgetData::new()
                },
                TextSettings {
                    size,
                    ..TextSettings::default()
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
