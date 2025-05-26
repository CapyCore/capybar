use std::rc::Rc;

use anyhow::Result;
use thiserror::Error;

use crate::{
    root::Environment,
    util::{Color, Drawer},
    widgets::{Widget, WidgetData, WidgetNew},
};

#[derive(Debug)]
pub enum Alignment {
    CenteringHorizontal,
    CenteringVertical,
    GrowthHorizontalRight(usize),
    GrowthHorizontalLeft(usize),
    GrowthVerticalUp(usize),
    GrowthVerticalDown(usize),
}

pub struct RowSettings {
    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
    pub alignment: Alignment,

    pub data: WidgetData,
}

impl RowSettings {
    pub fn default() -> Self {
        RowSettings {
            background: None,
            border: None,
            alignment: Alignment::GrowthHorizontalRight(10),

            data: WidgetData::default(),
        }
    }
}

#[derive(Error, Debug)]
pub enum RowError {
    #[error("Row is not wide enough to display all of it's child")]
    WidthOverflow,

    #[error("anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

pub struct Row {
    settings: RowSettings,

    children: Vec<Box<dyn Widget>>,
    env: Option<Rc<Environment>>,
}

impl Widget for Row {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.env = Some(Rc::clone(&env));
        let data = &mut self.settings.data;

        for child in &mut self.children {
            child.bind(Rc::clone(&env))?;
            let child_data = child.data()?;

            data.height = usize::max(
                data.height,
                child_data.height + child_data.position.1 + child_data.margin.3,
            );
        }

        self.align_children()?;

        Ok(())
    }

    fn draw(&mut self, drawer: &mut Drawer) -> Result<()> {
        if self.children.len() == 0 {
            self.settings.data.height =
                self.settings.border.unwrap_or_else(|| (5, Color::NONE)).0 * 3;
        }

        let border = match self.settings.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        let data = &self.settings.data;

        if let Some(color) = self.settings.background {
            for x in border.0..data.width - border.0 {
                for y in border.0..data.height - border.0 {
                    drawer.draw_pixel(&data, (x, y), color);
                }
            }
        }

        if let Some(color) = border.1 {
            for x in 0..border.0 {
                for y in 0..data.height {
                    drawer.draw_pixel(&data, (x, y), color);
                    drawer.draw_pixel(&data, (data.width - 1 - x, y), color);
                }
            }

            for x in 0..data.width {
                for y in 0..border.0 {
                    drawer.draw_pixel(&data, (x, y), color);
                    drawer.draw_pixel(&data, (x, data.height - 1 - y), color);
                }
            }
        }

        for widget in self.children.iter_mut() {
            widget.draw(drawer)?;
        }

        Ok(())
    }

    fn data(&mut self) -> Result<&mut WidgetData> {
        Ok(&mut self.settings.data)
    }
}

impl Row {
    pub fn add_child(&mut self, child: Box<dyn Widget>) -> Result<(), RowError> {
        self.children.push(child);

        match self.settings.alignment {
            Alignment::CenteringHorizontal => self.align_children_centered_horizontal()?,
            Alignment::CenteringVertical => todo!(),
            Alignment::GrowthHorizontalRight(_) => todo!(),
            Alignment::GrowthHorizontalLeft(_) => todo!(),
            Alignment::GrowthVerticalUp(_) => todo!(),
            Alignment::GrowthVerticalDown(_) => todo!(),
        }

        Ok(())
    }

    pub fn create_child<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W>,
    {
        self.add_child(Box::new(f(self.env.clone(), settings)?))?;
        Ok(())
    }

    fn align_children(&mut self) -> Result<(), RowError> {
        if self.children.len() == 0 {
            return Ok(());
        }

        match self.settings.alignment {
            Alignment::CenteringHorizontal => self.align_children_centered_horizontal()?,
            Alignment::CenteringVertical => todo!(),
            Alignment::GrowthHorizontalRight(_) => todo!(),
            Alignment::GrowthHorizontalLeft(_) => todo!(),
            Alignment::GrowthVerticalUp(_) => todo!(),
            Alignment::GrowthVerticalDown(_) => todo!(),
        };

        Ok(())
    }

    fn align_children_centered_horizontal(&mut self) -> Result<(), RowError> {
        let data = &mut self.settings.data;
        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut total_width = 0;
        for child in &mut self.children {
            total_width += {
                let data = child.data()?;
                data.width + data.margin.0 + data.margin.1
            }
        }

        if total_width > data.width - 2 * border {
            return Err(RowError::WidthOverflow);
        }

        if self.children.len() == 1 {
            let child = self.children[0].data()?;
            child.position.0 = child.position.0 + (data.width - border * 2 - total_width) / 2;
            child.position.1 = data.position.1 + border + child.margin.2;
            data.height = usize::max(
                data.height,
                child.height + child.position.1 + child.margin.3,
            );
            return Ok(());
        }

        let dist = (data.width - 2 * border - total_width) / (self.children.len() - 1);
        let mut x = data.position.0 + border;
        for child in self.children.iter_mut() {
            let child = child.data()?;
            child.position.0 = x + child.margin.0;

            child.position.1 = data.position.1 + border + child.margin.2;

            data.height = usize::max(
                data.height,
                child.height + child.position.1 + child.margin.3,
            );

            x += child.margin.0 + child.width + child.margin.1 + dist;
        }

        Ok(())
    }
}

impl WidgetNew for Row {
    type Settings = RowSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Row {
            settings,
            env,

            children: Vec::new(),
        })
    }
}
