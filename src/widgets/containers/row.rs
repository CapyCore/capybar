use crate::{
    util::{Color, Drawer},
    widgets::{Widget, WidgetData},
};

pub struct RowSettings {
    pub background: Option<Color>,
}

impl RowSettings {
    pub fn default() -> Self {
        RowSettings { background: None }
    }
}

pub struct Row {
    data: WidgetData,
    settings: RowSettings,

    children: Vec<Box<dyn Widget>>,
}

impl Widget for Row {
    fn draw(&mut self, drawer: &mut Drawer) {
        if let Some(color) = self.settings.background {
            for x in 0..self.data.width {
                for y in 0..self.data.height {
                    drawer.draw_pixel(&self.data, (x, y), color);
                }
            }
        }

        for widget in self.children.iter_mut() {
            widget.draw(drawer);
        }
    }

    fn data(&mut self) -> &mut WidgetData {
        &mut self.data
    }
}

impl Row {
    pub fn new(data: WidgetData, settings: Option<RowSettings>) -> Self {
        Row {
            data,
            settings: match settings {
                Some(a) => a,
                None => RowSettings::default(),
            },
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, mut child: Box<dyn Widget>) {
        let data = &mut child.data();
        data.position.0 = self.data.position.0 + self.data.width + data.margin.0;
        data.position.1 = self.data.position.1 + data.margin.2;
        self.data.width += data.width + data.margin.0 + data.margin.1;
        self.data.height = usize::max(
            self.data.height,
            data.height + data.margin.2 + data.margin.3,
        );
        self.children.push(child);
    }
}
