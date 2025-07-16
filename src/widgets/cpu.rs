use std::cell::RefCell;

use anyhow::Result;
use chrono::{DateTime, Local, TimeDelta};
use serde::Deserialize;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::{
    text::{Text, TextSettings},
    Style, Widget, WidgetData, WidgetError, WidgetNew,
};

/// Settings of a [CPU] widget
#[derive(Deserialize, Debug, Default, Clone)]
pub struct CPUSettings {
    #[serde(default, flatten)]
    pub default_data: WidgetData,

    /// Settings for underlying [Text] widget
    #[serde(default, flatten)]
    pub text_settings: TextSettings,

    #[serde(default, flatten)]
    pub style: Style,

    /// How often to update CPU status in milliseconds
    #[serde(default)]
    pub update_rate: u32,
}

/// Widget displaying current CPU status.
pub struct CPU {
    data: RefCell<WidgetData>,

    icon: RefCell<Text>,
    percent: RefCell<Text>,

    sys: RefCell<System>,

    last_update: RefCell<DateTime<Local>>,
    update_rate: TimeDelta,
}

impl CPU {
    fn get_info(&self) -> usize {
        let mut sys = self.sys.borrow_mut();
        sys.refresh_cpu_usage();
        sys.global_cpu_usage().round() as usize
    }

    fn align(&self) {
        let icon = self.icon.borrow_mut();
        let text = self.percent.borrow_mut();

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

impl Widget for CPU {
    fn bind(
        &mut self,
        env: std::rc::Rc<crate::root::Environment>,
    ) -> anyhow::Result<(), WidgetError> {
        self.percent.borrow_mut().bind(env.clone())?;
        self.icon.borrow_mut().bind(env)
    }

    fn init(&self) -> Result<(), WidgetError> {
        self.icon.borrow_mut().init()?;
        self.percent.borrow_mut().init()?;

        self.align();

        Ok(())
    }

    fn draw(&self) -> Result<(), WidgetError> {
        let mut last_update = self.last_update.borrow_mut();

        if Local::now() - *last_update >= self.update_rate {
            let info = self.get_info();

            {
                let mut text = self.percent.borrow_mut();
                if self.sys.borrow_mut().cpus().is_empty() {
                    self.icon.borrow_mut().change_text("");
                    text.change_text("ERR");
                } else {
                    text.change_text(format!("{info}%").as_str());
                }
            }

            self.align();
            *last_update = Local::now();
        }

        self.percent.borrow_mut().draw()?;
        self.icon.borrow_mut().draw()
    }

    fn data(&self) -> &RefCell<WidgetData> {
        &self.data
    }
}

impl WidgetNew for CPU {
    type Settings = CPUSettings;

    fn new(
        env: Option<std::rc::Rc<crate::root::Environment>>,
        settings: Self::Settings,
    ) -> Result<Self, WidgetError>
    where
        Self: Sized,
    {
        Ok(Self {
            data: RefCell::new(settings.default_data),
            icon: RefCell::new(Text::new(
                env.clone(),
                TextSettings {
                    text: "".to_string(),
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

            sys: RefCell::new(System::new_with_specifics(
                RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
            )),

            update_rate: TimeDelta::milliseconds(settings.update_rate as i64),
            last_update: RefCell::new(
                chrono::Local::now() - TimeDelta::milliseconds(settings.update_rate as i64),
            ),
        })
    }
}
