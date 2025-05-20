use std::{error::Error, fmt};

use crate::util::Drawer;

pub trait Widget {
    fn draw(&mut self, drawer: &mut Drawer);

    fn data(&mut self) -> &mut WidgetData;
}

#[derive(Debug)]
pub enum WidgetError {
    InvalidBounds,
}

impl Error for WidgetError {}

impl fmt::Display for WidgetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidBounds => write!(f, "Invalid widget bounds"),
        }
    }
}

pub struct WidgetData {
    pub position: (usize, usize),
    pub width: usize,
    pub height: usize,
    pub margin: (usize, usize, usize, usize),
}

impl WidgetData {
    pub fn new() -> Self {
        WidgetData {
            position: (0, 0),
            width: 0,
            height: 0,
            margin: (0, 0, 0, 0),
        }
    }
}
