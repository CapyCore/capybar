use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    root::Environment,
    widgets::{Style, Widget, WidgetData, WidgetNew},
};

use super::{
    container::Container,
    row::{Alignment, Row, RowSettings},
};

#[derive(Default, Debug, Clone, Deserialize)]
pub struct BarSettings {
    #[serde(flatten, default)]
    pub default_data: WidgetData,

    #[serde(default)]
    pub padding: (usize, usize, usize),

    #[serde(flatten)]
    pub style: Style,
}

impl BarSettings {
    pub const fn default() -> Self {
        Self {
            default_data: WidgetData::default(),
            padding: (10, 10, 10),
            style: Style::default(),
        }
    }
}

pub struct Bar {
    settings: BarSettings,
    data: RefCell<WidgetData>,
    env: Option<Rc<Environment>>,

    left: RefCell<Row>,
    center: RefCell<Row>,
    right: RefCell<Row>,
}

impl Bar {
    pub fn add_center(&self, widget: Box<dyn Widget>) -> Result<()> {
        self.center.borrow_mut().add_child(widget)?;

        Ok(())
    }

    pub fn create_child_left<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W>,
    {
        self.left
            .borrow_mut()
            .add_child(Box::new(f(self.env.clone(), settings)?))?;
        Ok(())
    }

    pub fn create_child_center<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W>,
    {
        self.center
            .borrow_mut()
            .add_child(Box::new(f(self.env.clone(), settings)?))?;
        Ok(())
    }

    pub fn create_child_right<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W>,
    {
        self.right
            .borrow_mut()
            .add_child(Box::new(f(self.env.clone(), settings)?))?;
        Ok(())
    }
}

impl Widget for Bar {
    fn bind(&mut self, env: std::rc::Rc<crate::root::Environment>) -> anyhow::Result<()> {
        self.left.borrow_mut().bind(Rc::clone(&env))?;
        self.center.borrow_mut().bind(Rc::clone(&env))?;
        self.right.borrow_mut().bind(Rc::clone(&env))
    }

    fn draw(&self, drawer: &mut crate::util::Drawer) -> anyhow::Result<()> {
        let data = self.data.borrow_mut();

        let border = match self.settings.style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        if let Some(color) = self.settings.style.background {
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

        {
            let left = self.left.borrow_mut();
            {
                let mut ld = left.data().borrow_mut();

                ld.position.0 = data.position.0 + border.0;
                ld.position.1 = data.position.1 + border.0;
            }

            left.draw(drawer)?;
        }

        {
            let center = self.center.borrow_mut();
            {
                let mut cd = center.data().borrow_mut();

                cd.position.0 = data.position.0 + (data.width - cd.width) / 2;
                cd.position.1 = data.position.1 + border.0;
            }
            center.draw(drawer)?;
        }

        {
            let right = self.right.borrow_mut();
            {
                let mut rd = right.data().borrow_mut();

                rd.position.0 = data.position.0 + data.width - border.0;
                rd.position.1 = data.position.1 + border.0;
            }
            right.draw(drawer)?;
        }

        Ok(())
    }

    fn init(&self) -> anyhow::Result<()> {
        let left = self.left.borrow_mut();
        let center = self.center.borrow_mut();
        let right = self.right.borrow_mut();
        left.init()?;
        center.init()?;
        right.init()?;

        let border = match self.settings.style.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

        let mut data = self.data.borrow_mut();
        data.height = *[
            left.data().borrow_mut().height,
            center.data().borrow_mut().height,
            right.data().borrow_mut().height,
        ]
        .iter()
        .max_by(|a, b| a.cmp(b))
        .unwrap()
            + 2 * border.0;

        Ok(())
    }

    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
    }
}

impl WidgetNew for Bar {
    type Settings = BarSettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            data: RefCell::new(settings.default_data),

            left: RefCell::new(Row::new(
                env.clone(),
                RowSettings {
                    alignment: Alignment::GrowthHorizontalRight(settings.padding.0),
                    ..RowSettings::default()
                },
            )?),

            center: RefCell::new(Row::new(
                env.clone(),
                RowSettings {
                    alignment: Alignment::GrowthCenteringHorizontalRight(settings.padding.1),
                    ..RowSettings::default()
                },
            )?),

            right: RefCell::new(Row::new(
                env.clone(),
                RowSettings {
                    alignment: Alignment::GrowthHorizontalLeft(settings.padding.2),
                    ..RowSettings::default()
                },
            )?),

            settings,
            env,
        })
    }
}

impl Container for Bar {
    fn align_children(&self) -> anyhow::Result<()> {
        todo!();
    }

    fn children(&self) -> &super::container::WidgetVec {
        todo!();
    }

    fn children_mut(&mut self) -> &super::container::WidgetVec {
        todo!();
    }
}
