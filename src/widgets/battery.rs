use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Add,
};

use anyhow::Result;
use battery::{Manager, State};
use serde::Deserialize;

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

/// Settings of a [Battery] widget
#[derive(Debug, Deserialize, Clone)]
pub struct BatterySettings {
    /// Array of all symbols for percentages of battery when it is not charging. Symbols are changed
    /// every 10% including 0%, therefor needs 11 symbols.  
    #[serde(default = "battery_not_charging_default")]
    pub battery_not_charging: [char; 11],

    /// Array of all symbols for percentages of battery when it is charging. Symbols are changed
    /// every 10% including 0%, therefor needs 11 symbols.  
    #[serde(default = "battery_charging_default")]
    pub battery_charging: [char; 11],

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
            battery_not_charging: battery_not_charging_default(),
            battery_charging: battery_charging_default(),

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
        }
    }
}

impl BatteryInfo {
    pub fn percentage(&self) -> f32 {
        self.energy / self.full
    }
}

/// Widget displaying current battery status.
pub struct Battery {
    manager: Manager,
    icon_text: RefCell<IconText>,

    settings: BatterySettings,
    data: RefCell<WidgetData>,
    is_ready: RefCell<bool>,

    prev_charge: RefCell<i8>,
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
                        Some(BatteryInfo {
                            energy: charge_rate * full,
                            full,
                            state: battery.state(),
                        })
                    }
                    Err(_) => None,
                })
                .fold(
                    BatteryInfo {
                        energy: 0.0,
                        full: 0.0,
                        state: battery::State::Unknown,
                    },
                    |acc, x| acc + x,
                ),
        )
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
            .change_icon(&self.settings.battery_not_charging[0].to_string());
        self.icon_text.borrow().init()?;

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

    fn draw(&self) -> anyhow::Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::Battery));
        }

        self.draw_style()?;

        let info = self.get_info();

        let mut prev_charge = self.prev_charge.borrow_mut();
        {
            let mut it = self.icon_text.borrow_mut();
            match info {
                Some(i) => {
                    let percentage: i8 = (i.percentage() * 100.0).round() as i8;

                    if percentage != *prev_charge {
                        it.change_icon(
                            format!(
                                "{}",
                                match i.state {
                                    State::Charging => self.settings.battery_charging,
                                    _ => self.settings.battery_not_charging,
                                }[(percentage / 10) as usize],
                            )
                            .as_str(),
                        );
                        it.change_text(format!("{percentage}%").as_str());
                    }
                }
                None => {
                    if *prev_charge != -1 {
                        it.change_icon("");
                        it.change_text("ERR");
                    }
                    *prev_charge = -1;
                }
            };
        }

        {
            let it = self.icon_text.borrow();
            let mut it_data = it.data_mut();
            let mut self_data = self.data.borrow_mut();
            it_data.position = self_data.position;
            self_data.width = it_data.width;
            self_data.height = it_data.height;
        }

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
            is_ready: RefCell::new(false),

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
            prev_charge: RefCell::new(0),
        })
    }
}

impl WidgetStyled for Battery {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}
