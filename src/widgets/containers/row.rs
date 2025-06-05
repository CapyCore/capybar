use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use thiserror::Error;

use crate::{
    root::Environment,
    util::{Color, Drawer},
    widgets::{Widget, WidgetData, WidgetNew},
};

use super::container::{Container, WidgetVec};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct RowSettings {
    pub background: Option<Color>,
    pub border: Option<(usize, Color)>,
    pub alignment: Alignment,

    pub default_data: WidgetData,
}

#[derive(Error, Debug)]
pub enum RowError {
    #[error("Row is not wide enough to display all of it's child")]
    WidthOverflow,

    #[error("anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

pub struct Row {
    settings: RowSettings,
    data: RefCell<WidgetData>,

    children: RefCell<WidgetVec>,
    env: Option<Rc<Environment>>,
}

impl Widget for Row {
    fn bind(&mut self, env: Rc<Environment>) -> Result<()> {
        self.env = Some(Rc::clone(&env));
        let mut children = self.children.borrow_mut();

        let children = children.widgets_mut();
        for child in children {
            child.bind(Rc::clone(&env))?;
        }

        Ok(())
    }

    fn init(&self) -> Result<()> {
        let mut data = self.data.borrow_mut();
        let mut children = self.children.borrow_mut();
        let border = match self.settings.border {
            Some(a) => a.0,
            None => 0,
        };

        let children = children.widgets_mut();
        for child in children {
            child.init()?;
            let child_data = child.data().borrow_mut();
            data.height = usize::max(
                data.height,
                child_data.height + child_data.position.1 + child_data.margin.3 + border,
            );
        }
        Ok(())
    }

    fn draw(&self, drawer: &mut Drawer) -> Result<()> {
        self.align_children()?;

        let children = self.children.borrow_mut();
        let mut data = self.data.borrow_mut();

        if children.is_empty() {
            data.height = self.settings.border.unwrap_or((5, Color::NONE)).0 * 3;
        }

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

        for widget in children.widgets() {
            widget.draw(drawer)?;
        }

        Ok(())
    }

    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
    }
}

impl Row {
    pub fn children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        self.children.get_mut().widgets_mut()
    }

    pub fn len(&self) -> usize {
        self.children.borrow().widgets().len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.borrow().widgets().is_empty()
    }

    pub fn pop(&mut self) {
        self.children.get_mut().widgets_mut().pop();
    }

    pub fn add_child(&mut self, child: Box<dyn Widget>) -> Result<(), RowError> {
        self.children.get_mut().widgets_mut().push(child);

        Ok(())
    }

    pub fn create_child<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W>,
    {
        self.add_child(Box::new(f(self.env.clone(), settings)?))?;
        Ok(())
    }

    fn get_max_height(children: &mut Vec<Box<dyn Widget>>) -> usize {
        if children.is_empty() {
            return 0;
        }

        let mut res = 0;
        for child in children.iter_mut().map(|a| a.data().borrow_mut()) {
            res = usize::max(res, child.height + child.position.1 + child.margin.3);
        }
        res
    }

    fn align_children_centered_horizontal(&self) -> Result<(), RowError> {
        let mut children = self.children.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        children.is_aligned = true;
        let children = children.widgets_mut();

        if children.len() == 1 {
            {
                let mut child = children[0].data().borrow_mut();

                child.position.0 = data.position.0 + (data.width - border * 2 - child.width) / 2;
                child.position.1 = data.position.1 + border + child.margin.2;
            }

            data.height = Row::get_max_height(children) + border;
            return Ok(());
        }

        let mut total_width = 0;
        for child in children.iter_mut() {
            total_width += {
                let data = child.data().borrow_mut();
                data.width + data.margin.0 + data.margin.1
            }
        }

        if total_width > data.width - 2 * border {
            return Err(RowError::WidthOverflow);
        }

        let dist = (data.width - 2 * border - total_width) / (children.len() - 1);
        let mut x = data.position.0 + border;

        for child in children.iter_mut() {
            let mut child = child.data().borrow_mut();

            child.position.0 = x + child.margin.0;
            child.position.1 = data.position.1 + child.margin.2;

            x += child.margin.0 + child.width + child.margin.1 + dist;
        }

        data.height = Row::get_max_height(children) + border;

        Ok(())
    }

    fn align_children_growth_ch(&self, padding: usize) -> Result<()> {
        {
            let mut children = self.children.borrow_mut();
            let mut data = self.data.borrow_mut();

            data.width = 0;

            for child in children
                .widgets_mut()
                .iter_mut()
                .map(|a| a.data().borrow_mut())
            {
                data.width += child.margin.0 + child.width + child.margin.1 + padding;
            }

            data.width -= padding;
        }

        self.align_children_centered_horizontal()?;

        Ok(())
    }

    fn align_children_growth_hr(&self, padding: usize) -> Result<()> {
        let mut children = self.children.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        children.is_aligned = true;
        let children = children.widgets_mut();

        let mut offset = border + data.position.0;
        for mut child in children.iter_mut().map(|a| a.data().borrow_mut()) {
            child.position.1 = data.position.1 + child.margin.2;
            child.position.0 = offset + child.margin.0;
            offset += child.margin.0 + child.width + child.margin.1 + padding;
        }

        data.width = offset - padding + border;

        Ok(())
    }

    fn align_children_growth_hl(&self, padding: usize) -> Result<()> {
        let mut children = self.children.borrow_mut();
        let mut data = self.data.borrow_mut();

        let border = match self.settings.border {
            Some((i, _)) => i,
            None => 0,
        };

        let children = children.widgets_mut();

        let mut offset = data.position.0 - border;
        for mut child in children.iter_mut().map(|a| a.data().borrow_mut()) {
            child.position.1 = data.position.1 + child.margin.2;
            child.position.0 = offset - child.width - child.margin.1;
            offset -= child.margin.0 + child.width + child.margin.1 + padding;
        }

        data.width = data.position.0 - (offset - padding + border);

        Ok(())
    }
}

impl WidgetNew for Row {
    type Settings = RowSettings;
    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            data: RefCell::new(settings.default_data),
            settings,
            env,
            children: RefCell::new(WidgetVec::new()),
        })
    }
}

impl Container for Row {
    fn align_children(&self) -> Result<()> {
        if self.children.borrow_mut().is_empty() {
            return Ok(());
        }

        match self.settings.alignment {
            Alignment::CenteringHorizontal => self.align_children_centered_horizontal()?,
            Alignment::CenteringVertical => todo!(),
            Alignment::GrowthCenteringHorizontalRight(padding) => {
                self.align_children_growth_ch(padding)?
            }
            Alignment::GrowthCenteringHorizontalLeft(_) => todo!(),
            Alignment::GrowthCenteringVerticalRight(_) => todo!(),
            Alignment::GrowthCenteringVerticalLeft(_) => todo!(),
            Alignment::GrowthHorizontalRight(padding) => self.align_children_growth_hr(padding)?,
            Alignment::GrowthHorizontalLeft(padding) => self.align_children_growth_hl(padding)?,
            Alignment::GrowthVerticalUp(_) => todo!(),
            Alignment::GrowthVerticalDown(_) => todo!(),
        };

        Ok(())
    }

    fn children(&self) -> &WidgetVec {
        todo!();
    }

    fn children_mut(&mut self) -> &WidgetVec {
        todo!();
    }
}
