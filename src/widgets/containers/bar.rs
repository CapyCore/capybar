use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    root::Environment,
    services::Service,
    widgets::{Style, Widget, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled},
};

use super::{
    row::{Alignment, Row, RowSettings},
    Container,
};

/// Settings of a [Bar] containert
#[derive(Default, Debug, Clone, Deserialize)]
pub struct BarSettings {
    #[serde(flatten, default)]
    pub default_data: WidgetData,

    /// Distance between widgets in underlying rows. Stored as a tuple of (Distance in left,
    /// distance in center, distance in right)
    #[serde(default)]
    pub padding: (usize, usize, usize),

    #[serde(default)]
    pub left_settings: RowSettings,

    #[serde(default)]
    pub center_settings: RowSettings,

    #[serde(default)]
    pub right_settings: RowSettings,

    #[serde(flatten)]
    pub style: Style,
}

impl BarSettings {
    pub const fn default() -> Self {
        Self {
            default_data: WidgetData::default(),
            padding: (10, 10, 10),
            left_settings: RowSettings::default(),
            center_settings: RowSettings::default(),
            right_settings: RowSettings::default(),
            style: Style::default(),
        }
    }
}

/// Main widget in capybar. Stores 3 alligned [Row] containers.
pub struct Bar {
    settings: BarSettings,
    data: RefCell<WidgetData>,
    env: Option<Rc<Environment>>,

    left: RefCell<Row>,
    center: RefCell<Row>,
    right: RefCell<Row>,
    services: RefCell<Vec<Box<dyn Service>>>,
}

impl Bar {
    pub fn add_center(&self, widget: Box<dyn Widget>) -> Result<()> {
        self.center.borrow_mut().add_widget(widget);

        Ok(())
    }

    pub fn create_widget_left<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, WidgetError>,
    {
        self.left
            .borrow_mut()
            .add_widget(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    pub fn create_widget_center<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, WidgetError>,
    {
        self.center
            .borrow_mut()
            .add_widget(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    pub fn create_widget_right<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, WidgetError>,
    {
        self.right
            .borrow_mut()
            .add_widget(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    pub fn left(&mut self) -> &mut RefCell<Row> {
        &mut self.left
    }

    pub fn center(&mut self) -> &mut RefCell<Row> {
        &mut self.center
    }

    pub fn right(&mut self) -> &mut RefCell<Row> {
        &mut self.right
    }

    fn align_widgets(&self) -> anyhow::Result<()> {
        let mut data = self.data.borrow_mut();
        let border = match self.settings.style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        let left = self.left.borrow_mut();
        let mut ld = left.data_mut();

        ld.position.0 = data.position.0 + border.0;
        ld.position.1 = data.position.1 + border.0;

        let center = self.center.borrow_mut();
        let mut cd = center.data_mut();

        cd.position.0 = data.position.0 + (data.width - cd.width) / 2;
        cd.position.1 = data.position.1 + border.0;

        let right = self.right.borrow_mut();
        let mut rd = right.data_mut();

        rd.position.0 = data.position.0 + data.width - border.0;
        rd.position.1 = data.position.1 + border.0;

        data.height = ld.height.max(cd.height).max(rd.height);

        Ok(())
    }
}

impl Widget for Bar {
    fn name(&self) -> WidgetList {
        WidgetList::Bar
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

    fn bind(
        &mut self,
        env: std::rc::Rc<crate::root::Environment>,
    ) -> anyhow::Result<(), WidgetError> {
        self.left.borrow_mut().bind(Rc::clone(&env))?;
        self.center.borrow_mut().bind(Rc::clone(&env))?;
        self.right.borrow_mut().bind(Rc::clone(&env))?;

        for service in self.services.borrow_mut().iter_mut() {
            if let Err(e) = service.bind(Rc::clone(&env)) {
                return Err(WidgetError::Custom(e.into()));
            }
        }
        self.env = Some(env);
        Ok(())
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.env.clone()
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        self.left.borrow().prepare()?;
        self.center.borrow().prepare()?;
        self.right.borrow().prepare()?;

        self.align_widgets()?;
        self.apply_style()?;

        Ok(())
    }

    fn draw(&self) -> anyhow::Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Bar));
        }

        self.draw_style()?;

        self.left.borrow_mut().draw()?;
        self.center.borrow_mut().draw()?;
        self.right.borrow_mut().draw()?;

        Ok(())
    }

    fn init(&self) -> Result<(), WidgetError> {
        let left = self.left.borrow_mut();
        let center = self.center.borrow_mut();
        let right = self.right.borrow_mut();
        left.init()?;
        center.init()?;
        right.init()?;
        right.data_mut().position.0 = self.data().width;

        let border = match self.settings.style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        let mut data = self.data_mut();
        data.height = *[
            left.data_mut().height,
            center.data_mut().height,
            right.data_mut().height,
        ]
        .iter()
        .max_by(|a, b| a.cmp(b))
        .unwrap()
            + 2 * border.0;

        Ok(())
    }
}

impl WidgetNew for Bar {
    type Settings = BarSettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        Ok(Self {
            data: RefCell::new(settings.default_data),

            left: RefCell::new(Row::new(
                env.clone(),
                RowSettings {
                    alignment: Alignment::GrowthHorizontalRight(settings.padding.0),
                    ..settings.left_settings
                },
            )?),

            center: RefCell::new(Row::new(
                env.clone(),
                RowSettings {
                    alignment: Alignment::GrowthCenteringHorizontalRight(settings.padding.1),
                    ..settings.center_settings
                },
            )?),

            right: RefCell::new(Row::new(
                env.clone(),
                RowSettings {
                    alignment: Alignment::GrowthHorizontalLeft(settings.padding.2),
                    ..settings.right_settings
                },
            )?),
            services: RefCell::new(Vec::new()),

            settings,
            env,
        })
    }
}

impl WidgetStyled for Bar {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}

impl Container for Bar {
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
        self.left.borrow().run()?;
        self.center.borrow().run()?;
        self.right.borrow().run()?;
        Ok(())
    }
}
