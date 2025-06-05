use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use thiserror::Error;

use crate::{
    root::Environment,
    util::{Color, Drawer},
};

pub trait Widget {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()>;
    fn draw(&self, drawer: &mut Drawer) -> Result<()>;
    fn init(&self) -> Result<()>;
    fn data(&self) -> &RefCell<WidgetData>;
}

pub trait WidgetNew: Widget {
    type Settings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Error)]
pub enum WidgetError {
    #[error("Invalid widget bounds")]
    InvalidBounds,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WidgetData {
    pub position: (usize, usize),
    pub width: usize,
    pub height: usize,
    pub margin: (usize, usize, usize, usize),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Style {
    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
}

pub trait WidgetStyled: Widget {
    fn style(&self) -> &Style;

    fn style_mut(&mut self) -> &mut Style;

    fn apply_style(&self) -> Result<()> {
        let mut data = self.data().borrow_mut();
        let style = self.style();

        let border = match style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        data.height += border.0 * 2;

        Ok(())
    }
}
