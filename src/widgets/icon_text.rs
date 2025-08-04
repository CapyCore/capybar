use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use serde::Deserialize;

use crate::root::Environment;

use super::{
    text::{Text, TextSettings},
    Margin, Style, Widget, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled,
};

#[derive(Default, Debug, Clone, Deserialize)]
pub struct IconTextSettings {
    #[serde(default, flatten)]
    pub default_data: WidgetData,

    #[serde(default)]
    pub text_settings: TextSettings,

    #[serde(default)]
    pub icon_settings: TextSettings,

    #[serde(default)]
    pub style: Style,
}

pub struct IconText {
    data: RefCell<WidgetData>,
    env: Option<Rc<Environment>>,
    settings: IconTextSettings,

    icon: Text,
    text: Text,

    is_ready: RefCell<bool>,
}

impl IconText {
    fn align(&self) {
        let mut icon_data = self.icon.data_mut();
        let icon_style = self.icon.style();

        let mut text_data = self.text.data_mut();
        let text_style = self.text.style();
        let data = &mut self.data.borrow_mut();

        icon_data.position.0 = data.position.0 + icon_style.margin.left;
        icon_data.position.1 = data.position.1 + icon_style.margin.up;
        text_data.position.0 = icon_data.position.0
            + icon_data.width
            + icon_style.margin.right
            + text_style.margin.left;
        text_data.position.1 = data.position.1 + text_style.margin.up;

        data.height = usize::max(
            text_data.position.1 - data.position.1 + text_data.height + text_style.margin.down,
            icon_data.position.1 - data.position.1 + icon_data.height + icon_style.margin.down,
        );

        data.width = icon_style.margin.left
            + icon_style.margin.right
            + icon_data.width
            + text_style.margin.left
            + text_style.margin.right
            + text_data.width;
    }

    pub fn change_text(&mut self, text: &str) {
        self.text.change_text(text);
    }

    pub fn change_icon(&mut self, text: &str) {
        self.icon.change_text(text);
    }
}

impl Widget for IconText {
    fn name(&self) -> WidgetList {
        WidgetList::IconText
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
        self.text.bind(env.clone())?;
        self.icon.bind(env.clone())?;
        self.env = Some(env);
        Ok(())
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.env.clone()
    }

    fn init(&self) -> Result<(), WidgetError> {
        self.icon.init()?;
        self.text.init()?;

        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        self.text.prepare()?;
        self.icon.prepare()?;

        self.align();
        self.apply_style()?;
        *self.is_ready.borrow_mut() = true;
        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::IconText));
        }

        if !*self.is_ready.borrow() {
            self.prepare()?;
        }

        self.draw_style()?;
        let style = self.style();
        self.icon.data_mut().position += (style.margin.left, style.margin.up);
        self.icon.data_mut().position += (style.margin.left, style.margin.up);

        self.text.draw()?;
        self.icon.draw()
    }
}

impl WidgetNew for IconText {
    type Settings = IconTextSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        Ok(Self {
            data: RefCell::new(settings.default_data),

            icon: Text::new(
                env.clone(),
                TextSettings {
                    default_data: WidgetData {
                        ..WidgetData::default()
                    },
                    style: Style {
                        margin: Margin {
                            left: 2,
                            right: 0,
                            up: 0,
                            down: 0,
                        },
                        ..Style::default()
                    },
                    fontid: 1,
                    ..settings.text_settings.clone()
                },
            )?,

            text: Text::new(
                env.clone(),
                TextSettings {
                    default_data: WidgetData {
                        ..WidgetData::default()
                    },
                    style: Style {
                        margin: Margin {
                            left: 2,
                            right: 2,
                            up: 1,
                            down: 0,
                        },
                        ..Style::default()
                    },
                    ..settings.text_settings.clone()
                },
            )?,

            env,
            settings,

            is_ready: RefCell::new(false),
        })
    }
}

impl WidgetStyled for IconText {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}
