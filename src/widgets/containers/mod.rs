pub mod bar;
pub mod row;

use std::borrow::{Borrow, BorrowMut};

use anyhow::Result;

use crate::widgets::Widget;

/// Wrapper for [Vec]. Currently is pretty useles though....
pub struct WidgetVec {
    pub is_aligned: bool,
    pub widgets: Vec<Box<dyn Widget>>,
}

impl Default for WidgetVec {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetVec {
    pub fn new() -> Self {
        Self {
            is_aligned: false,
            widgets: Vec::new(),
        }
    }

    pub fn is_aligned(&self) -> bool {
        self.is_aligned
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    pub fn widgets(&self) -> &Vec<Box<dyn Widget>> {
        self.borrow()
    }

    pub fn widgets_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        self.borrow_mut()
    }
}

impl Borrow<Vec<Box<dyn Widget>>> for WidgetVec {
    fn borrow(&self) -> &Vec<Box<dyn Widget>> {
        &self.widgets
    }
}

impl BorrowMut<Vec<Box<dyn Widget>>> for WidgetVec {
    fn borrow_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        self.is_aligned = false;
        &mut self.widgets
    }
}

/// Container is a widget that is responsible for positioning of it's child widgets. It may or may
/// not have any additional logic behind it.
pub trait Container: Widget {
    /// Align is called before drawing the container
    fn align_children(&self) -> Result<()>;

    fn children(&self) -> &WidgetVec;

    fn children_mut(&mut self) -> &WidgetVec;
}
