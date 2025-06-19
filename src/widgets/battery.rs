use std::{cell::RefCell, ops::Add};

use anyhow::Result;
use battery::{Manager, State};
use serde::Deserialize;

use super::{
    text::{Text, TextSettings},
    Style, Widget, WidgetData, WidgetNew, WidgetStyled,
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

    icon: RefCell<Text>,
    percent: RefCell<Text>,
    settings: BatterySettings,
    data: RefCell<WidgetData>,

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

    fn align(&self) -> Result<()> {
        let icon = self.icon.borrow_mut();
        let mut icon_data = icon.data().borrow_mut();
        let percent = self.percent.borrow_mut();
        let mut percent_data = percent.data().borrow_mut();

        let data = &mut self.data.borrow_mut();

        icon_data.position.0 = data.position.0 + icon_data.margin.0;
        icon_data.position.1 = data.position.1 + icon_data.margin.2;
        percent_data.position.0 =
            icon_data.position.0 + icon_data.width + icon_data.margin.1 + percent_data.margin.0;
        percent_data.position.1 = data.position.1 + percent_data.margin.2;

        data.height = usize::max(
            percent_data.position.1 + percent_data.height + percent_data.margin.3,
            icon_data.position.1 + icon_data.height + icon_data.margin.3,
        );

        data.width = icon_data.margin.0
            + icon_data.margin.1
            + icon_data.width
            + percent_data.margin.0
            + percent_data.margin.1
            + percent_data.width;

        Ok(())
    }
}

impl Widget for Battery {
    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
    }

    fn bind(&mut self, env: std::rc::Rc<crate::root::Environment>) -> anyhow::Result<()> {
        self.percent.borrow_mut().bind(env.clone())?;
        self.icon.borrow_mut().bind(env)
    }

    fn init(&self) -> Result<()> {
        self.icon.borrow_mut().init()?;
        self.percent.borrow_mut().init()?;

        self.apply_style()?;
        self.align()?;

        Ok(())
    }

    fn draw(&self, drawer: &mut crate::util::Drawer) -> anyhow::Result<()> {
        let info = self.get_info();

        let mut prev_charge = self.prev_charge.borrow_mut();
        {
            let mut text = self.percent.borrow_mut();
            let mut icon = self.icon.borrow_mut();
            match info {
                Some(i) => {
                    let percentage: i8 = (i.percentage() * 100.0).round() as i8;

                    if percentage != *prev_charge {
                        icon.change_text(
                            format!(
                                "{}",
                                match i.state {
                                    State::Charging => self.settings.battery_charging,
                                    _ => self.settings.battery_not_charging,
                                }[(percentage / 10) as usize],
                            )
                            .as_str(),
                        );
                        text.change_text(format!("{percentage}%").as_str());
                    }
                }
                None => {
                    if *prev_charge != -1 {
                        self.icon.borrow_mut().change_text("");
                        text.change_text("ERR");
                    }
                    *prev_charge = -1;
                }
            };
        }

        self.align()?;
        self.percent.borrow_mut().draw(drawer)?;
        self.icon.borrow_mut().draw(drawer)
    }
}

impl WidgetNew for Battery {
    type Settings = BatterySettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            manager: Manager::new()?,

            icon: RefCell::new(Text::new(
                env.clone(),
                TextSettings {
                    text: "󰂎".to_string(),
                    default_data: WidgetData {
                        margin: (0, 0, 0, 0),
                        ..WidgetData::default()
                    },
                    fontid: 1,
                    ..settings.text_settings.clone()
                },
            )?),
            percent: RefCell::new(Text::new(
                env,
                TextSettings {
                    text: "Err".to_string(),

                    default_data: WidgetData {
                        margin: (5, 0, 2, 0),
                        ..WidgetData::default()
                    },
                    ..settings.text_settings.clone()
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

    fn style_mut(&mut self) -> &mut Style {
        &mut self.settings.style
    }
}
