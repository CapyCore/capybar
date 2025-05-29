use std::cell::RefCell;

use anyhow::{Ok, Result};
use hyprland::{
    data::Workspace,
    shared::{HyprData, HyprDataVec},
};

use crate::widgets::text::{Text, TextSettings};

use super::{
    containers::row::{Row, RowSettings},
    Widget, WidgetData, WidgetNew,
};

pub enum WorkspacesError {}

#[derive(Default)]
pub struct WorkspacesSettings {
    pub data: WidgetData,

    pub text: TextSettings,
    pub row: RowSettings,
}

pub struct Workspaces {
    settings: WorkspacesSettings,
    workspaces: RefCell<Vec<Workspace>>,
    row: RefCell<Row>,
}

impl Workspaces {
    fn init(&self) -> Result<()> {
        let mut workspaces = self.workspaces.borrow_mut();
        *workspaces = hyprland::data::Workspaces::get()?.to_vec();

        workspaces
            .sort_by(|first, second| first.id.cmp(&second.id));

        let mut row = self.row.borrow_mut();
        for w in workspaces.iter_mut() {
            row.create_child(
                Text::new,
                TextSettings {
                    text: w.name.clone(),
                    ..self.settings.text
                },
            )?;
        }

        Ok(())

    }

    fn rearange(&self) -> Result<()> {
        todo!();
        Ok(())
    }
}

impl Widget for Workspaces {
    fn data(&mut self) -> anyhow::Result<&mut super::WidgetData> {
        Ok(&mut self.settings.data)
    }

    fn bind(&mut self, env: std::rc::Rc<crate::root::Environment>) -> anyhow::Result<()> {
        self.init()?;
        let mut row = self.row.borrow_mut();
        let settings = row.data()?;
        settings.position.0 += settings.margin.0;
        settings.position.1 += settings.margin.2;
        row.bind(env)
    }

    fn draw(&self, drawer: &mut crate::util::Drawer) -> anyhow::Result<()> {
        //self.rearange()?;
        self.row.borrow().draw(drawer)
    }
}

impl WidgetNew for Workspaces {
    type Settings = WorkspacesSettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Workspaces {
            row: RefCell::new(Row::new(env, settings.row)?),
            workspaces: RefCell::new(Vec::new()),
            settings,
        })
    }
}
