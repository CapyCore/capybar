use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    root::Environment,
    services::Service,
    util::Color,
    widgets::{Style, Widget, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled},
};

use super::{Container, ContainerSingle};

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(tag = "type", content = "padding")]
pub enum Alignment {
    CenteringHorizontal,
    CenteringVertical,
    GrowthCenteringHorizontalRight(usize),
    GrowthCenteringHorizontalLeft(usize),
    GrowthCenteringVerticalRight(usize),
    GrowthCenteringVerticalLeft(usize),
    GrowthHorizontalRight(usize),
    GrowthHorizontalLeft(usize),
    GrowthVerticalUp(usize),
    GrowthVerticalDown(usize),
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::GrowthHorizontalRight(10)
    }
}

impl Alignment {
    pub const fn default() -> Self {
        Alignment::GrowthHorizontalRight(10)
    }
}

/// Settings of a [Row] container
#[derive(Default, Deserialize, Debug, Clone, Copy)]
pub struct RowSettings {
    #[serde(default)]
    pub alignment: Alignment,

    #[serde(default, flatten)]
    pub default_data: WidgetData,
    #[serde(default, flatten)]
    pub style: Style,
}

impl RowSettings {
    pub const fn default() -> RowSettings {
        RowSettings {
            alignment: Alignment::default(),
            default_data: WidgetData::default(),
            style: Style::default(),
        }
    }
}

#[derive(Error, Debug)]
pub enum RowError {
    #[error("Row is not wide enough to display all of it's widgets")]
    WidthOverflow,

    #[error("anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Container that stores widgets in a row.
pub struct Row {
    settings: RowSettings,
    data: RefCell<WidgetData>,

    widgets: RefCell<Vec<Box<dyn Widget>>>,
    env: Option<Rc<Environment>>,
    services: RefCell<Vec<Box<dyn Service>>>,

    is_ready: RefCell<bool>,
}

impl Widget for Row {
    fn name(&self) -> WidgetList {
        WidgetList::Row
    }

    fn as_styled(&self) -> Option<&dyn WidgetStyled> {
        Some(self)
    }

    fn data(&self) -> Ref<'_, WidgetData> {
        self.data.borrow()
    }

    fn data_mut(&self) -> RefMut<'_, WidgetData> {
        self.data.borrow_mut()
    }

    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError> {
        self.env = Some(Rc::clone(&env));

        let mut widgets = self.widgets.borrow_mut();
        for widget in widgets.iter_mut() {
            widget.bind(Rc::clone(&env))?;
        }

        for service in self.services.borrow_mut().iter_mut() {
            if let Err(e) = service.bind(Rc::clone(&env)) {
                return Err(WidgetError::Custom(e.into()));
            }
        }

        Ok(())
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.env.clone()
    }

    fn init(&self) -> Result<(), WidgetError> {
        let mut data = self.data.borrow_mut();

        let border = match self.settings.style.border {
            Some(a) => a.0,
            None => 0,
        };

        let widgets = self.widgets.borrow();
        for widget in widgets.iter() {
            widget.init()?;
            let widget_data = widget.data();
            data.height = usize::max(
                data.height,
                widget_data.height
                    + widget_data.position.1
                    + border
                    + self.settings.style.margin.up
                    + self.settings.style.margin.down,
            );
        }
        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        for widget in self.widgets.borrow_mut().iter() {
            widget.prepare()?;
        }

        self.align_widgets()?;
        self.apply_style()?;

        *self.is_ready.borrow_mut() = true;
        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Row));
        }

        if !*self.is_ready.borrow() {
            self.prepare()?;
        }
        *self.is_ready.borrow_mut() = false;

        self.draw_style()?;

        for widget in self.widgets.borrow_mut().iter() {
            widget.draw()?;
        }

        Ok(())
    }
}

impl Row {
    pub fn widgets_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        self.widgets.get_mut()
    }

    pub fn len(&self) -> usize {
        self.widgets.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.borrow().is_empty()
    }

    pub fn pop(&mut self) {
        self.widgets.get_mut().pop();
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.get_mut().push(widget);
    }

    fn get_max_height(widgets: &mut Vec<Box<dyn Widget>>) -> usize {
        if widgets.is_empty() {
            return 0;
        }

        let mut res = 0;
        for widget in widgets.iter_mut().map(|a| a.data()) {
            res = usize::max(res, widget.height + widget.position.1);
        }
        res
    }

    fn align_widgets_centered_horizontal(&self) -> Result<(), RowError> {
        let mut data = self.data.borrow_mut();

        let border = match self.settings.style.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut widgets = self.widgets.borrow_mut();

        if widgets.len() == 1 {
            {
                let mut widget = widgets[0].data_mut();

                widget.position.0 = data.position.0
                    + (data.width - border * 2 - widget.width) / 2
                    + self.style().margin.left;
                widget.position.1 = data.position.1 + border + self.style().margin.up;
                if let Some(styled) = widgets[0].as_styled() {
                    widget.position.1 += styled.style().margin.up;
                }
            }

            data.height = Row::get_max_height(&mut widgets) + border;
            return Ok(());
        }

        let mut total_width = 0;
        for widget in widgets.iter_mut() {
            total_width += widget.data_mut().width;
        }

        if total_width > data.width - 2 * border {
            return Err(RowError::WidthOverflow);
        }

        let dist = (data.width - 2 * border - total_width) / (widgets.len() - 1);
        let mut x = data.position.0 + border;

        for widget in widgets.iter_mut() {
            let mut widget = widget.data_mut();

            widget.position.0 = x;
            widget.position.1 = data.position.1;

            x += widget.width + dist;
        }

        data.height = Row::get_max_height(&mut widgets) + border;

        Ok(())
    }

    fn align_widgets_growth_ch(&self, padding: usize) -> Result<()> {
        {
            let mut widgets = self.widgets.borrow_mut();
            let mut data = self.data.borrow_mut();

            data.width = 0;

            for widget in widgets.iter_mut().map(|a| a.data_mut()) {
                data.width += widget.width + padding;
            }

            data.width -= padding;
        }

        self.align_widgets_centered_horizontal()?;

        Ok(())
    }

    fn align_widgets_growth_hr(&self, padding: usize) -> Result<()> {
        let mut widgets = self.widgets.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.style.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut offset = border + data.position.0 + self.settings.style.margin.left;
        data.height = 0;
        for mut widget in widgets.iter_mut().map(|a| a.data_mut()) {
            widget.position.1 = data.position.1 + self.settings.style.margin.up + border;
            widget.position.0 = offset;
            offset += widget.width + padding;
            data.height = usize::max(data.height, widget.height);
        }

        data.width = offset - padding + border;
        data.height += self.settings.style.margin.up + self.settings.style.margin.down + 2 * border;

        Ok(())
    }

    fn align_widgets_growth_hl(&self, padding: usize) -> Result<()> {
        let mut widgets = self.widgets.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.style.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut offset = data.position.0 - border - self.settings.style.margin.right;
        data.height = 0;
        for mut widget in widgets.iter_mut().map(|a| a.data_mut()) {
            widget.position.1 = data.position.1;
            widget.position.0 = offset - widget.width;
            offset -= widget.width + padding;
            data.height = usize::max(data.height, widget.height);
        }
        data.height += self.settings.style.margin.up + self.settings.style.margin.down + 2 * border;

        data.width = data.position.0 + padding - offset - border;

        data.position.0 -= data.width;

        Ok(())
    }

    fn align_widgets(&self) -> Result<()> {
        if self.widgets.borrow_mut().is_empty() {
            self.data.borrow_mut().height =
                self.settings.style.border.unwrap_or((5, Color::NONE)).0 * 3;
            return Ok(());
        }

        match self.settings.alignment {
            Alignment::CenteringHorizontal => self.align_widgets_centered_horizontal()?,
            Alignment::CenteringVertical => todo!(),
            Alignment::GrowthCenteringHorizontalRight(padding) => {
                self.align_widgets_growth_ch(padding)?
            }
            Alignment::GrowthCenteringHorizontalLeft(_) => todo!(),
            Alignment::GrowthCenteringVerticalRight(_) => todo!(),
            Alignment::GrowthCenteringVerticalLeft(_) => todo!(),
            Alignment::GrowthHorizontalRight(padding) => self.align_widgets_growth_hr(padding)?,
            Alignment::GrowthHorizontalLeft(padding) => self.align_widgets_growth_hl(padding)?,
            Alignment::GrowthVerticalUp(_) => todo!(),
            Alignment::GrowthVerticalDown(_) => todo!(),
        };

        Ok(())
    }
}

impl WidgetNew for Row {
    type Settings = RowSettings;
    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        Ok(Self {
            data: RefCell::new(settings.default_data),
            settings,
            env,
            widgets: RefCell::new(Vec::new()),
            services: RefCell::new(Vec::new()),
            is_ready: RefCell::new(false),
        })
    }
}

impl WidgetStyled for Row {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}

impl Container for Row {
    fn create_service<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: crate::services::ServiceNew + crate::services::Service + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, crate::services::ServiceError>,
    {
        self.services
            .borrow_mut()
            .push(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    fn run(&self) -> Result<()> {
        for service in self.services.borrow_mut().iter() {
            service.run()?;
        }

        Ok(())
    }
}

impl ContainerSingle for Row {
    fn create_widget<W, F>(&mut self, f: F, settings: W::Settings) -> Result<(), WidgetError>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, WidgetError>,
    {
        self.add_widget(Box::new(f(self.env.clone(), settings)?));

        Ok(())
    }
}
