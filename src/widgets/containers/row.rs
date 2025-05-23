use std::{error::Error, fmt::Display};

use crate::{
    util::{Color, Drawer},
    widgets::{Widget, WidgetData},
};

pub enum Alignment {
    CenteringHorizontal,
    GrowthHorizontal(usize),
    GrowthVertical(usize),
}

pub struct RowSettings {
    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
    pub alignment: Alignment,
}

impl RowSettings {
    pub fn default() -> Self {
        RowSettings {
            background: None,
            border: None,
            alignment: Alignment::GrowthHorizontal(10),
        }
    }
}

#[derive(Debug)]
pub enum RowError {
    WidthOverflow,
}

impl Error for RowError {}

impl Display for RowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WidthOverflow => write!(f, "Row is not wide enough to display all of it's child"),
        }
    }
}

pub struct Row {
    data: WidgetData,
    settings: RowSettings,

    children: Vec<Box<dyn Widget>>,
}

impl Widget for Row {
    fn draw(&mut self, drawer: &mut Drawer) {
        let border = match self.settings.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        if let Some(color) = self.settings.background {
            for x in border.0..self.data.width - border.0 {
                for y in border.0..self.data.height - border.0 {
                    drawer.draw_pixel(&self.data, (x, y), color);
                }
            }
        }

        if let Some(color) = border.1 {
            for x in 0..border.0 {
                for y in 0..self.data.height {
                    drawer.draw_pixel(&self.data, (x, y), color);
                    drawer.draw_pixel(&self.data, (self.data.width - 1 - x, y), color);
                }
            }

            for x in 0..self.data.width {
                for y in 0..border.0 {
                    drawer.draw_pixel(&self.data, (x, y), color);
                    drawer.draw_pixel(&self.data, (x, self.data.height - 1 - y), color);
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
    pub fn new(mut data: WidgetData, settings: Option<RowSettings>) -> Self {
        Row {
            settings: match settings {
                Some(a) => {
                    if let Some((i, _)) = a.border {
                        data.width += i * 2;
                        data.height += i * 2;
                    }
                    a
                }
                None => RowSettings::default(),
            },
            data,

            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Box<dyn Widget>) -> Result<(), RowError> {
        match self.settings.alignment {
            Alignment::CenteringHorizontal => self.grow_child_centered_horizontal(child)?,
            Alignment::GrowthHorizontal(padding) => self.grow_child_horizontal(child, padding),
            Alignment::GrowthVertical(padding) => self.grow_child_vertical(child, padding),
        };

        Ok(())
    }

    fn grow_child_centered_horizontal(
        &mut self,
        mut child: Box<dyn Widget>,
    ) -> Result<(), RowError> {
        let data = &mut child.data();
        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        data.position.1 = self.data.position.1 + border + data.margin.2;
        self.data.height = usize::max(
            self.data.height,
            data.height + data.position.1 + data.margin.3,
        );

        self.children.push(child);

        let total_width = self.children.iter_mut().fold(0, |acc, e| {
            acc + {
                let data = e.data();
                data.width + data.margin.0 + data.margin.1
            }
        });

        if total_width > self.data.width - 2 * border {
            return Err(RowError::WidthOverflow);
        }

        if self.children.len() == 1 {
            let data = self.children[0].data();
            data.position.0 = self.data.position.0 + (self.data.width - border * 2 - total_width) / 2;
            return Ok(())
        }

        let dist = (self.data.width - 2*border - total_width) / (self.children.len()-1);
        let mut x = self.data.position.0 + border;
        for child in self.children.iter_mut() {
            let data = child.data();

            data.position.0 = x + data.margin.0;

            x += data.margin.0 + data.width + data.margin.1 + dist;
        }

        Ok(())
    }

    fn grow_child_horizontal(&mut self, mut child: Box<dyn Widget>, padding: usize) {
        let data = &mut child.data();
        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        data.position.0 = self.data.position.0 + self.data.width + data.margin.0;
        if self.children.len() != 0 {
            data.position.0 += padding;
        }
        data.position.1 = self.data.position.1 + data.margin.2 + border;
        self.data.width = data.position.0 + data.width + data.margin.1 + border;
        self.data.height = usize::max(
            self.data.height,
            data.height + data.position.1 + data.margin.3,
        );
        self.children.push(child);
    }

    fn grow_child_vertical(&mut self, mut child: Box<dyn Widget>, padding: usize) {
        let data = &mut child.data();
        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        data.position.1 = self.data.position.1 + self.data.height + data.margin.2;
        if self.children.len() != 0 {
            data.position.1 += padding;
        }
        data.position.0 = self.data.position.0 + data.margin.0 + border;
        self.data.height = data.position.1 + data.height + data.margin.3 + border;
        self.data.width = usize::max(
            self.data.height,
            data.width + data.position.0 + data.margin.1,
        );
        self.children.push(child);
    }
}
