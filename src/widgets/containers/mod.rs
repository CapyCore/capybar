pub mod bar;
pub mod row;

use std::rc::Rc;

use anyhow::Result;

use crate::{
    root::Environment,
    services::{Service, ServiceError, ServiceNew},
    widgets::Widget,
};

use super::{WidgetError, WidgetNew};

/// [Container] is a [Widget] that is responsible for positioning of it's child widgets. It may or may
/// not have any additional logic behind it.
pub trait Container: Widget {
    fn create_service<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: ServiceNew + Service + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, ServiceError>;

    /// Run all child [Service] objects
    fn run(&self) -> Result<()>;
}

/// Trait that describes [Container] that has single clear way to add child widget
pub trait ContainerSingle: Container {
    fn create_widget<W, F>(&mut self, f: F, settings: W::Settings) -> Result<(), WidgetError>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, WidgetError>;
}
