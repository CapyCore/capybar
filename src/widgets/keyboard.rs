use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    root::Environment, services::ServiceList, util::signals::SignalNames, widgets::Widget,
};

use super::{
    icon_text::{IconText, IconTextSettings},
    text::TextSettings,
    Style, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled,
};

/// Settings of a [Keyboard] widget
#[derive(Deserialize, Default, Debug, Clone)]
pub struct KeyboardSettings {
    #[serde(default, flatten)]
    pub default_data: WidgetData,

    /// Settings for underlying [Text] widget
    #[serde(default, flatten)]
    pub text_settings: TextSettings,

    #[serde(default, flatten)]
    pub style: Style,

    /// Map from underlying layout name to display name
    #[serde(default)]
    pub layout_mappings: HashMap<String, String>,
}

/// Widget displaying current keyboard layout.
pub struct Keyboard {
    data: RefCell<WidgetData>,
    style: Style,
    is_ready: RefCell<bool>,

    layout_mappings: Rc<HashMap<String, String>>,

    icon_text: Rc<RefCell<IconText>>,

    env: Option<Rc<Environment>>,
}

impl Widget for Keyboard {
    fn name(&self) -> WidgetList {
        WidgetList::Keyboard
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
        self.env = Some(env.clone());
        self.icon_text.borrow_mut().bind(env)
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.env.clone()
    }

    fn init(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::InitWithNoEnv(WidgetList::Keyboard));
        }

        let signals = self.env.as_ref().unwrap().signals.borrow_mut();

        if !signals.contains_key(&SignalNames::Keyboard) {
            return Err(WidgetError::NoCorespondingSignal(
                WidgetList::Keyboard,
                ServiceList::Keyboard,
            ));
        }

        let signal_ic = Rc::clone(&self.icon_text);
        let layout_mappings = Rc::clone(&self.layout_mappings);

        signals[&SignalNames::Keyboard].connect(move |data| {
            if let Some(text) = data.downcast_ref::<String>() {
                let layout = if layout_mappings.contains_key(text) {
                    layout_mappings.get(text).unwrap()
                } else {
                    &text.to_string()
                };

                signal_ic.borrow_mut().change_text(layout);
            }
        });

        {
            let mut ic = self.icon_text.borrow_mut();
            ic.change_icon("󰌌");
            ic.change_text("ERR");
            ic.init()?;
        }

        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        {
            let it = self.icon_text.borrow();
            it.prepare()?;
            let mut it_data = it.data_mut();
            let mut self_data = self.data.borrow_mut();
            it_data.position = self_data.position;
            self_data.width = it_data.width;
            self_data.height = it_data.height;
        }

        self.apply_style()?;

        *self.is_ready.borrow_mut() = true;
        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Keyboard));
        }

        if !*self.is_ready.borrow() {
            self.prepare()?;
        }

        self.draw_style()?;

        {
            let ic_data = self.icon_text.borrow();
            ic_data.data_mut().position = self.data().position;
        }
        self.icon_text.borrow_mut().draw()
    }
}

impl WidgetNew for Keyboard {
    type Settings = KeyboardSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        Ok(Keyboard {
            data: RefCell::new(settings.default_data),
            style: settings.style,
            is_ready: RefCell::new(false),

            layout_mappings: Rc::new(settings.layout_mappings),

            icon_text: Rc::new(RefCell::new(IconText::new(
                env.clone(),
                IconTextSettings {
                    icon_settings: settings.text_settings.clone(),
                    text_settings: settings.text_settings.clone(),
                    ..IconTextSettings::default()
                },
            )?)),

            env: None,
        })
    }
}

impl WidgetStyled for Keyboard {
    fn style(&self) -> &Style {
        &self.style
    }
}
