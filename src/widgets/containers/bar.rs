use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{
    root::Environment,
    util::Color,
    widgets::{Widget, WidgetData, WidgetNew},
};

use super::{
    container::Container,
    row::{Alignment, Row, RowSettings},
};

pub struct BarSettings {
    pub default_data: WidgetData,

    pub padding: (usize, usize, usize),

    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
}

impl Default for BarSettings {
    fn default() -> Self {
        Self {
            default_data: WidgetData::default(),
            padding: (10, 10, 10),
            background: None,
            border: None,
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

        let border = match self.settings.border {
            Some(a) => (a.0, Some(a.1)),
            None => (0, None),
        };

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

        self.left.borrow_mut().draw(drawer)?;

        {
            let mut center = self.center.borrow_mut();
            let cd = center.data()?;

            cd.position.0 = data.position.0 + (data.width - cd.width) / 2;

            center.draw(drawer)?;
        }

        {
            let mut right = self.right.borrow_mut();
            right.data()?.position.0 = data.position.0 + data.width;

            right.draw(drawer)?;
        }

        Ok(())
    }

    fn init(&self) -> anyhow::Result<()> {
        let mut left = self.left.borrow_mut();
        let mut center = self.center.borrow_mut();
        let mut right = self.right.borrow_mut();
        left.init()?;
        center.init()?;
        right.init()?;

        let mut data = self.data.borrow_mut();
        data.height = *[
            5,
            left.data()?.height,
            center.data()?.height,
            right.data()?.height,
        ]
        .iter()
        .max_by(|a, b| a.cmp(b))
        .unwrap();

        Ok(())
    }

    fn data(&mut self) -> anyhow::Result<&mut crate::widgets::WidgetData> {
        Ok(self.data.get_mut())
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
