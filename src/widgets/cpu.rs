use std::cell::{Ref, RefCell, RefMut};

use anyhow::Result;
use chrono::{DateTime, Local, TimeDelta};
use serde::Deserialize;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::{
    icon_text::{IconText, IconTextSettings},
    text::TextSettings,
    Style, Widget, WidgetData, WidgetError, WidgetList, WidgetNew, WidgetStyled,
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
    settings: CPUSettings,
    is_ready: RefCell<bool>,

    icon_text: RefCell<IconText>,

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
}

impl Widget for CPU {
    fn name(&self) -> WidgetList {
        WidgetList::CPU
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

    fn bind(
        &mut self,
        env: std::rc::Rc<crate::root::Environment>,
    ) -> anyhow::Result<(), WidgetError> {
        self.icon_text.borrow_mut().bind(env)
    }

    fn env(&self) -> Option<std::rc::Rc<crate::root::Environment>> {
        self.icon_text.borrow().env()
    }

    fn init(&self) -> Result<(), WidgetError> {
        self.apply_style()?;

        self.icon_text.borrow_mut().change_text("Err");
        self.icon_text.borrow_mut().change_icon("");
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

    fn draw(&self) -> Result<(), WidgetError> {
        if self.env().is_none() {
            return Err(WidgetError::DrawWithNoEnv(WidgetList::CPU));
        }

        self.draw_style()?;

        let mut last_update = self.last_update.borrow_mut();

        if Local::now() - *last_update >= self.update_rate {
            let info = self.get_info();

            if self.sys.borrow_mut().cpus().is_empty() {
                self.icon_text.borrow_mut().change_icon("");
                self.icon_text.borrow_mut().change_text("ERR");
            } else {
                self.icon_text
                    .borrow_mut()
                    .change_text(format!("{info}%").as_str());
            }

            *last_update = Local::now();
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

            is_ready: RefCell::new(false),

            icon_text: RefCell::new(IconText::new(
                env.clone(),
                IconTextSettings {
                    icon_settings: settings.text_settings.clone(),
                    text_settings: settings.text_settings.clone(),
                    ..IconTextSettings::default()
                },
            )?),

            sys: RefCell::new(System::new_with_specifics(
                RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
            )),

            update_rate: TimeDelta::milliseconds(settings.update_rate as i64),
            last_update: RefCell::new(
                chrono::Local::now() - TimeDelta::milliseconds(settings.update_rate as i64),
            ),

            settings,
        })
    }
}

impl WidgetStyled for CPU {
    fn style(&self) -> &Style {
        &self.settings.style
    }
}
