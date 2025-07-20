use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use thiserror::Error;

use crate::{
    root::Environment,
    services::Service,
    util::Color,
    widgets::{Widget, WidgetData, WidgetError, WidgetNew},
};

use super::{Container, ContainerSingle};

#[derive(Debug, Clone, Copy)]
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

/// Settings of a [Row] container
#[derive(Debug, Default, Clone, Copy)]
pub struct RowSettings {
    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
    pub alignment: Alignment,

    pub default_data: WidgetData,
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
}

impl Widget for Row {
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

    fn init(&self) -> Result<(), WidgetError> {
        let mut data = self.data.borrow_mut();
        let border = match self.settings.border {
            Some(a) => a.0,
            None => 0,
        };

        let widgets = self.widgets.borrow();
        for widget in widgets.iter() {
            widget.init()?;
            let widget_data = widget.data().borrow_mut();
            data.height = usize::max(
                data.height,
                widget_data.height + widget_data.position.1 + widget_data.margin.3 + border,
            );
        }
        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::DrawWithNoEnv("Row".to_string()));
        }

        self.align_widgets()?;

        let widgets = self.widgets.borrow_mut();
        let mut data = self.data.borrow_mut();

        if widgets.is_empty() {
            data.height = self.settings.border.unwrap_or((5, Color::NONE)).0 * 3;
        }

        let border = match self.settings.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        {
            let mut drawer = self.env.as_ref().unwrap().drawer.borrow_mut();
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
        }

        for widget in widgets.iter() {
            widget.draw()?;
        }

        Ok(())
    }

    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
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
        for widget in widgets.iter_mut().map(|a| a.data().borrow_mut()) {
            res = usize::max(res, widget.height + widget.position.1 + widget.margin.3);
        }
        res
    }

    fn align_widgets_centered_horizontal(&self) -> Result<(), RowError> {
        let mut data = self.data.borrow_mut();

        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut widgets = self.widgets.borrow_mut();

        if widgets.len() == 1 {
            {
                let mut widget = widgets[0].data().borrow_mut();

                widget.position.0 = data.position.0 + (data.width - border * 2 - widget.width) / 2;
                widget.position.1 = data.position.1 + border + widget.margin.2;
            }

            data.height = Row::get_max_height(&mut widgets) + border;
            return Ok(());
        }

        let mut total_width = 0;
        for widget in widgets.iter_mut() {
            total_width += {
                let data = widget.data().borrow_mut();
                data.width + data.margin.0 + data.margin.1
            }
        }

        if total_width > data.width - 2 * border {
            return Err(RowError::WidthOverflow);
        }

        let dist = (data.width - 2 * border - total_width) / (widgets.len() - 1);
        let mut x = data.position.0 + border;

        for widget in widgets.iter_mut() {
            let mut widget = widget.data().borrow_mut();

            widget.position.0 = x + widget.margin.0;
            widget.position.1 = data.position.1 + widget.margin.2;

            x += widget.margin.0 + widget.width + widget.margin.1 + dist;
        }

        data.height = Row::get_max_height(&mut widgets) + border;

        Ok(())
    }

    fn align_widgets_growth_ch(&self, padding: usize) -> Result<()> {
        {
            let mut widgets = self.widgets.borrow_mut();
            let mut data = self.data.borrow_mut();

            data.width = 0;

            for widget in widgets.iter_mut().map(|a| a.data().borrow_mut()) {
                data.width += widget.margin.0 + widget.width + widget.margin.1 + padding;
            }

            data.width -= padding;
        }

        self.align_widgets_centered_horizontal()?;

        Ok(())
    }

    fn align_widgets_growth_hr(&self, padding: usize) -> Result<()> {
        let mut widgets = self.widgets.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut offset = border + data.position.0;
        for mut widget in widgets.iter_mut().map(|a| a.data().borrow_mut()) {
            widget.position.1 = data.position.1 + widget.margin.2;
            widget.position.0 = offset + widget.margin.0;
            offset += widget.margin.0 + widget.width + widget.margin.1 + padding;
        }

        data.width = offset - padding + border;

        Ok(())
    }

    fn align_widgets_growth_hl(&self, padding: usize) -> Result<()> {
        let mut widgets = self.widgets.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        let mut offset = data.position.0 - border;
        for mut widget in widgets.iter_mut().map(|a| a.data().borrow_mut()) {
            widget.position.1 = data.position.1 + widget.margin.2;
            widget.position.0 = offset - widget.width - widget.margin.1;
            offset -= widget.margin.0 + widget.width + widget.margin.1 + padding;
        }

        data.width = data.position.0 - (offset - padding + border);

        Ok(())
    }

    fn align_widgets(&self) -> Result<()> {
        if self.widgets.borrow_mut().is_empty() {
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
        })
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
