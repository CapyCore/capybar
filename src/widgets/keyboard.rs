use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
use serde::Deserialize;

use crate::{
    root::Environment,
    widgets::{text::Text, Widget},
};

use super::{text::TextSettings, Style, WidgetData, WidgetError, WidgetNew};

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
    layout_mappings: Rc<HashMap<String, String>>,

    icon: RefCell<Text>,
    text: Rc<RefCell<Text>>,

    env: Option<Rc<Environment>>,
}

impl Keyboard {
    fn align(&self) {
        let icon = self.icon.borrow_mut();
        let text = self.text.borrow_mut();

        let mut icon_data = icon.data().borrow_mut();
        let mut text_data = text.data().borrow_mut();
        let data = &mut self.data.borrow_mut();

        icon_data.position.0 = data.position.0 + icon_data.margin.0;
        icon_data.position.1 = data.position.1 + icon_data.margin.2;
        text_data.position.0 =
            icon_data.position.0 + icon_data.width + icon_data.margin.1 + text_data.margin.0;
        text_data.position.1 = data.position.1 + text_data.margin.2;

        data.height = usize::max(
            text_data.position.1 + text_data.height + text_data.margin.3,
            icon_data.position.1 + icon_data.height + icon_data.margin.3,
        );

        data.width = icon_data.margin.0
            + icon_data.margin.1
            + icon_data.width
            + text_data.margin.0
            + text_data.margin.1
            + text_data.width;
    }
}

impl Widget for Keyboard {
    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError> {
        self.env = Some(env.clone());
        self.text.borrow_mut().bind(env.clone())?;
        self.icon.borrow_mut().bind(env)
    }

    fn init(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::InitWithNoEnv("Keyboard".to_string()));
        }

        let signals = self.env.as_ref().unwrap().signals.borrow_mut();

        if !signals.contains_key("keyboard") {
            return Err(WidgetError::NoCorespondingSignal(
                "Keyboard".to_string(),
                "Keyboard".to_string(),
            ));
        }

        let signal_text = Rc::clone(&self.text);
        let layout_mappings = Rc::clone(&self.layout_mappings);

        signals["keyboard"].connect(move |data| {
            if let Some(text) = data.downcast_ref::<String>() {
                let layout = if layout_mappings.contains_key(text) {
                    layout_mappings.get(text).unwrap()
                } else {
                    &text.to_string()
                };

                signal_text.borrow_mut().change_text(layout);
            }
        });

        self.icon.borrow_mut().init()?;
        self.text.borrow_mut().init()?;

        self.align();

        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        self.align();

        self.text.borrow_mut().draw()?;
        self.icon.borrow_mut().draw()
    }

    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
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
            layout_mappings: Rc::new(settings.layout_mappings),

            icon: RefCell::new(Text::new(
                env.clone(),
                TextSettings {
                    text: "󰌌".to_string(),
                    default_data: WidgetData {
                        margin: (0, 0, 0, 0),
                        ..WidgetData::default()
                    },
                    fontid: 1,
                    ..settings.text_settings.clone()
                },
            )?),
            text: Rc::new(RefCell::new(Text::new(
                env,
                TextSettings {
                    text: String::new(),

                    default_data: WidgetData {
                        margin: (5, 0, 2, 0),
                        ..WidgetData::default()
                    },
                    ..settings.text_settings.clone()
                },
            )?)),

            env: None,
        })
    }
}
