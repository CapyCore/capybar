use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{self, Debug},
    rc::Rc,
};

use anyhow::Result;

use serde::Deserialize;

use smithay_client_toolkit::seat::pointer::PointerEventKind;
use thiserror::Error;

use crate::{root::Environment, util::Color, widgets::Widget};

use super::{
    text::{Text, TextSettings},
    Margin, Style, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled,
};

/// Callback function wrapper for Button widget interactions.
///
/// Provides a way to execute custom code when a button is pressed.
/// The callback receives a reference to the Button that triggered it.
///
/// # Examples
/// ```
/// use capybar::widgets::button::ButtonCallback;
/// use std::{cell::RefCell, rc::Rc};
///
/// // Track button press count
/// let counter = Rc::new(RefCell::new(0));
/// let counter_clone = Rc::clone(&counter);
///
/// let callback = ButtonCallback::new(move |button| {
///     *counter_clone.borrow_mut() += 1;
///     println!("Button pressed {} times", counter_clone.borrow());
///     Ok(())
/// });
/// ```
#[derive(Clone)]
pub struct ButtonCallback(Rc<dyn Fn(&Button) -> Result<()>>);

impl ButtonCallback {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Button) -> Result<()> + 'static,
    {
        ButtonCallback(Rc::new(f))
    }

    pub fn call(&self, button: &Button) -> Result<()> {
        (self.0)(button)
    }
}

impl Default for ButtonCallback {
    fn default() -> Self {
        ButtonCallback::new(|_| Ok(()))
    }
}

impl Debug for ButtonCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ButtonCallback(..)")
    }
}

/// Configuration settings for a [Button] widget.
///
/// Defines the appearance, behavior, and interactive properties of button widgets.
/// Supports customizable text, fonts, callback actions, and visual states for
/// hover and press interactions.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ButtonSettings {
    /// Default widget data including position and size information
    #[serde(default, flatten)]
    pub default_data: WidgetData,

    /// Font size for the button text, in points
    #[serde(default)]
    pub size: f32,

    /// Callback function executed when the button is pressed.
    /// This field is skipped during deserialization and must be set programmatically
    #[serde(skip)]
    pub callback: ButtonCallback,

    /// Background color displayed when the mouse hovers over the button.
    /// Overrides the regular background color from style when the mouse is over the button
    #[serde(default)]
    pub hover_background: Color,

    /// Background color displayed when the button is pressed down.
    /// Takes priority over hover_background and regular background while the button is held
    #[serde(default)]
    pub press_background: Color,

    /// Additional text-specific settings including text content, color, and font properties
    #[serde(default, flatten)]
    pub text_settings: TextSettings,

    /// Common styling settings including margins, borders, and default background color
    #[serde(default, flatten)]
    pub style: Style,
}

#[derive(Debug, Error)]
pub enum ButtonError {}

/// Interactive button widget that responds to mouse events.
///
/// Displays text and supports visual feedback for different mouse interaction states:
/// - Regular state: Uses background color from style settings
/// - Hover state: Uses hover_background color when mouse is over the button
/// - Press state: Uses press_background color when button is held down
///
/// Executes a configured callback function when pressed. Text content can be
/// dynamically changed at runtime using change_text().
pub struct Button {
    settings: ButtonSettings,
    data: RefCell<WidgetData>,
    env: Option<Rc<Environment>>,

    is_pressed: RefCell<bool>,
    is_hovered: RefCell<bool>,

    text: RefCell<Text>,
}

impl Button {
    fn align(&self) {
        let text = self.text.borrow();
        let mut text_data = text.data_mut();
        let text_style = text.style();

        let mut self_data = self.data.borrow_mut();

        text_data.position.0 = self_data.position.0 + text_style.margin.left;
        text_data.position.1 = self_data.position.1 + text_style.margin.up;

        self_data.height =
            text_data.position.1 - self_data.position.1 + text_data.height + text_style.margin.down;

        self_data.width = text_style.margin.left + text_style.margin.right + text_data.width;
    }

    /// Text is not cached as a string and gets consturcted every time. Often usage of the function might be pricy.
    pub fn get_text(&self) -> String {
        self.text.borrow().get_text()
    }

    pub fn change_text(&self, text: &str) {
        self.text.borrow_mut().change_text(text);
    }
}

impl Widget for Button {
    fn name(&self) -> WidgetList {
        WidgetList::Button
    }

    fn as_styled(&self) -> Option<&dyn WidgetStyled> {
        Some(self)
    }

    fn bind(&mut self, env: Rc<Environment>) -> Result<(), WidgetError> {
        self.env = Some(env.clone());
        self.text.borrow_mut().bind(env)
    }

    fn handle_mouse_event(
        &self,
        event: &smithay_client_toolkit::seat::pointer::PointerEvent,
    ) -> Result<(), WidgetError> {
        use PointerEventKind::*;
        match &event.kind {
            Press { button, .. } => {
                if *button == 272 {
                    *self.is_pressed.borrow_mut() = true;
                    self.settings.callback.call(self)?;
                }
            }
            Release { button, .. } => {
                if *button == 272 {
                    *self.is_pressed.borrow_mut() = false;
                }
            }
            Enter { .. } => *self.is_hovered.borrow_mut() = true,
            Leave { .. } => *self.is_hovered.borrow_mut() = false,
            _ => {}
        }

        Ok(())
    }

    fn env(&self) -> Option<Rc<Environment>> {
        self.env.clone()
    }

    fn init(&self) -> Result<(), WidgetError> {
        {
            self.text.borrow_mut().init()?;
        }
        self.align();
        self.data.borrow_mut().height = self.text.borrow_mut().data().height as usize;

        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        {
            self.text.borrow_mut().prepare()?;
        }
        self.align();

        self.apply_style()?;

        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env.is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Button));
        }

        self.draw_style()?;
        self.text.borrow_mut().draw()?;

        Ok(())
    }

    fn data(&self) -> Ref<'_, WidgetData> {
        self.data.borrow()
    }

    fn data_mut(&self) -> RefMut<'_, WidgetData> {
        self.data.borrow_mut()
    }
}

impl WidgetNew for Button {
    type Settings = ButtonSettings;

    fn new(env: Option<Rc<Environment>>, settings: Self::Settings) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        let button = Button {
            data: RefCell::new(settings.default_data),

            text: RefCell::new(Text::new(
                env.clone(),
                TextSettings {
                    default_data: WidgetData {
                        ..WidgetData::default()
                    },
                    style: Style {
                        margin: Margin {
                            left: 2,
                            right: 2,
                            up: 0,
                            down: 0,
                        },
                        ..Style::default()
                    },
                    fontid: 0,
                    ..settings.text_settings.clone()
                },
            )?),

            is_pressed: RefCell::new(false),
            is_hovered: RefCell::new(false),

            env,
            settings: settings,
        };

        Ok(button)
    }
}

impl WidgetStyled for Button {
    fn style(&self) -> &Style {
        &self.settings.style
    }

    fn draw_style(&self) -> Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(self.name()));
        }

        let env = self.env().unwrap();
        let style = self.style();
        let border = style.border.unwrap_or((0, Color::NONE));

        let mouse_pos = *env.mouse_position.borrow();
        let data_bounds = {
            let current_data = self.data();
            (
                current_data.position.0 + style.margin.left,
                current_data.position.1 + style.margin.up,
                current_data.width,
                current_data.height,
            )
        };
        let is_hovering = mouse_pos.0 >= data_bounds.0 as f64
            && mouse_pos.0 < (data_bounds.0 + data_bounds.2) as f64
            && mouse_pos.1 >= data_bounds.1 as f64
            && mouse_pos.1 < (data_bounds.1 + data_bounds.3) as f64;

        *self.is_hovered.borrow_mut() = is_hovering;

        let mut data = self.data_mut();
        data.position.0 += style.margin.left;
        data.position.1 += style.margin.up;

        let mut drawer = env.as_ref().drawer.borrow_mut();

        if let Some(mut color) = style.background {
            if *self.is_pressed.borrow() {
                color = self.settings.press_background;
            } else if is_hovering {
                color = self.settings.hover_background;
            }
            for x in border.0..data.width - border.0 {
                for y in border.0..data.height - border.0 {
                    drawer.draw_pixel(&data, (x, y), color);
                }
            }
        }

        if border.1 == Color::NONE {
            return Ok(());
        }

        for x in 0..border.0 {
            for y in 0..data.height {
                drawer.draw_pixel(&data, (x, y), border.1);
                drawer.draw_pixel(&data, (data.width - 1 - x, y), border.1);
            }
        }

        for x in 0..data.width {
            for y in 0..border.0 {
                drawer.draw_pixel(&data, (x, y), border.1);
                drawer.draw_pixel(&data, (x, data.height - 1 - y), border.1);
            }
        }

        Ok(())
    }
}
