use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Add,
    time::Duration,
};

use anyhow::Result;
use battery::{units::time::nanosecond, Manager, State};
use serde::Deserialize;
use smithay_client_toolkit::seat::pointer::PointerEventKind;

use super::{
    icon_text::{IconText, IconTextSettings},
    text::TextSettings,
    Style, Widget, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled,
};

const fn battery_not_charging_default() -> [char; 11] {
    ['󰂎', '󰁺', '󰁻', '󰁼', '󰁽', '󰁾', '󰁿', '󰂀', '󰂁', '󰂂', '󰁹']
}

const fn battery_charging_default() -> [char; 11] {
    ['󰢟', '󰢜', '󰂆', '󰂇', '󰂈', '󰢝', '󰂉', '󰢞', '󰂊', '󰂋', '󰂅']
}

const fn battery_state_default() -> BatteryState {
    BatteryState::Percentage
}

/// Settings of a [Battery] widget
#[derive(Debug, Deserialize, Clone)]
pub struct BatterySettings {
    /// Array of all symbols for percentages of battery when it is not charging. Symbols are changed
    /// every 10% including 0%, therefor needs 11 symbols.  
    #[serde(default = "battery_not_charging_default")]
    pub battery_discharging: [char; 11],

    /// Array of all symbols for percentages of battery when it is charging. Symbols are changed
    /// every 10% including 0%, therefor needs 11 symbols.  
    #[serde(default = "battery_charging_default")]
    pub battery_charging: [char; 11],

    /// Default state of a widget. Controls type of displayed iformation. Could be either "Percentage" or "Time"
    #[serde(default = "battery_state_default")]
    pub default_state: BatteryState,

    /// Settings for underlying [Text] widget
    #[serde(default, flatten)]
    pub text_settings: TextSettings,

    #[serde(default, flatten)]
    pub default_data: WidgetData,

    #[serde(default, flatten)]
    pub style: Style,
}

impl Default for BatterySettings {
    fn default() -> Self {
        Self {
            battery_discharging: battery_not_charging_default(),
            battery_charging: battery_charging_default(),
            default_state: battery_state_default(),

            text_settings: TextSettings::default(),

            default_data: WidgetData::default(),

            style: Style::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct BatteryInfo {
    energy: f32,
    full: f32,
    state: State,
    time: Duration,
}

impl Add for BatteryInfo {
    type Output = BatteryInfo;
    fn add(self, rhs: Self) -> Self::Output {
        BatteryInfo {
            energy: self.energy + rhs.energy,
            full: self.full + rhs.full,
            state: {
                if self.state == State::Charging || rhs.state == State::Charging {
                    State::Charging
                } else if self.state == State::Discharging || rhs.state == State::Discharging {
                    State::Discharging
                } else if self.state == rhs.state {
                    self.state
                } else if self.state == State::Unknown {
                    rhs.state
                } else if rhs.state == State::Unknown {
                    self.state
                } else {
                    State::Unknown
                }
            },
            time: self.time + rhs.time,
        }
    }
}

impl BatteryInfo {
    pub fn percentage(&self) -> f32 {
        self.energy / self.full
    }
}

/// Posible states of [Battery] widget
#[derive(Clone, Copy, Debug, Deserialize)]
pub enum BatteryState {
    Percentage,
    Time,
}

/// Widget displaying current battery status. Changes displayed information on click. Can dispaly
/// time to charge/discarg or precentage.
pub struct Battery {
    manager: Manager,
    icon_text: RefCell<IconText>,

    settings: BatterySettings,
    data: RefCell<WidgetData>,

    state: RefCell<BatteryState>,
}

impl Battery {
    /// Get information of current battery status
    pub fn get_info(&self) -> Option<BatteryInfo> {
        Some(
            self.manager
                .batteries()
                .ok()?
                .filter_map(|battery| match battery {
                    Ok(battery) => {
                        let charge_rate = battery.state_of_charge().value;
                        let full = battery.energy_full().value;
                        let state = battery.state();
                        use State::*;
                        let time = match state {
                            Charging => {
                                Duration::from_nanos(if let Some(value) = battery.time_to_full() {
                                    value.get::<nanosecond>() as u64
                                } else {
                                    0
                                })
                            }
                            _ => {
                                Duration::from_nanos(if let Some(value) = battery.time_to_empty() {
                                    value.get::<nanosecond>() as u64
                                } else {
                                    0
                                })
                            }
                        };
                        Some(BatteryInfo {
                            energy: charge_rate * full,
                            full,
                            state,
                            time,
                        })
                    }
                    Err(_) => None,
                })
                .fold(
                    BatteryInfo {
                        energy: 0.0,
                        full: 0.0,
                        state: battery::State::Unknown,
                        time: Duration::from_nanos(0),
                    },
                    |acc, x| acc + x,
                ),
        )
    }

    fn update_text(&self) {
        use BatteryState::*;
        let info = self.get_info();
        let mut it = self.icon_text.borrow_mut();
        if info.is_none() {
            it.change_icon("");
            it.change_text("ERR");
        }
        let info = info.unwrap();
        let percentage: i8 = (info.percentage() * 100.0).round() as i8;
        it.change_icon(
            format!(
                "{}",
                match info.state {
                    State::Charging => self.settings.battery_charging,
                    _ => self.settings.battery_discharging,
                }[(percentage / 10) as usize],
            )
            .as_str(),
        );
        match *self.state.borrow() {
            Percentage => {
                it.change_text(format!("{percentage: >2}%").as_str());
            }
            Time => {
                let time = info.time.as_secs();
                it.change_text(format!("{:02}:{:02}", time / 3600, time / 60 % 60).as_str());
            }
        }
    }
}

impl Widget for Battery {
    fn name(&self) -> WidgetList {
        WidgetList::Battery
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

    fn handle_mouse_press(
        &self,
        event: &smithay_client_toolkit::seat::pointer::PointerEvent,
    ) -> Result<(), WidgetError> {
        if let PointerEventKind::Press { button, .. } = event.kind {
            if button == 272 {
                let mut state = self.state.borrow_mut();
                use BatteryState::*;
                *state = match *state {
                    Time => Percentage,
                    Percentage => Time,
                };
            }
        }

        Ok(())
    }

    fn env(&self) -> Option<std::rc::Rc<crate::root::Environment>> {
        self.icon_text.borrow().env()
    }

    fn bind(
        &mut self,
        env: std::rc::Rc<crate::root::Environment>,
    ) -> anyhow::Result<(), WidgetError> {
        self.icon_text.borrow_mut().bind(env)
    }

    fn init(&self) -> Result<(), WidgetError> {
        self.apply_style()?;

        self.icon_text.borrow_mut().change_text("Err");
        self.icon_text
            .borrow_mut()
            .change_icon(&self.settings.battery_discharging[0].to_string());
        self.icon_text.borrow().init()?;

        Ok(())
    }

    fn prepare(&self) -> Result<(), WidgetError> {
        self.update_text();

        {
            let it = self.icon_text.borrow();
            let mut it_data = it.data_mut();
            let mut self_data = self.data.borrow_mut();
            it_data.position = self_data.position;
            self_data.width = it_data.width;
            self_data.height = it_data.height;
        }

        self.apply_style()?;
        self.icon_text.borrow().prepare()?;

        Ok(())
    }

    fn draw(&self) -> anyhow::Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Battery));
        }

        self.draw_style()?;

        self.icon_text.borrow().draw()
    }
}

impl WidgetNew for Battery {
    type Settings = BatterySettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        let manager = Manager::new();
        if let Err(err) = manager {
            return Err(WidgetError::Custom(err.into()));
        }

        let manager = manager.unwrap();
        Ok(Self {
            manager,

            icon_text: RefCell::new(IconText::new(
                env.clone(),
                IconTextSettings {
                    icon_settings: settings.text_settings.clone(),
                    text_settings: settings.text_settings.clone(),
                    ..IconTextSettings::default()
                },
            )?),

            data: RefCell::new(settings.default_data),
            settings,

            state: RefCell::new(BatteryState::Percentage),
        })
    }
}

impl WidgetStyled for Battery {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}
