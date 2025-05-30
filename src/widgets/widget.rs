use std::rc::Rc;

use anyhow::Result;
use thiserror::Error;

use crate::{root::Environment, util::Drawer};

pub trait Widget {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()>;

    fn draw(&self, drawer: &mut Drawer) -> Result<()>;

    fn data(&mut self) -> Result<&mut WidgetData>;
}

pub trait WidgetNew {
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
