use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    num::NonZeroU32,
    rc::Rc,
    thread,
    time::Duration,
};

use anyhow::Result;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use thiserror::Error;
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, EventQueue, QueueHandle,
};

use crate::{
    config::Config,
    processes::{clients, Process, ProcessError, ProcessNew},
    util::{
        fonts::{self, FontsError},
        signals::Signal,
        Drawer,
    },
    widgets::{
        self, battery::Battery, clock::Clock, containers::bar::Bar, cpu::CPU, keyboard::Keyboard,
        text::Text, Widget, WidgetError, WidgetNew, WidgetsList,
    },
};

/// Structure containing things all the widgets in capybar needs access to
pub struct Environment {
    pub config: Config,
    pub drawer: RefCell<Drawer>,
    pub signals: RefCell<HashMap<String, Signal>>,
}

#[derive(Error, Debug)]
pub enum RootError {
    #[error("Environment is not initialised before drawing")]
    EnvironmentNotInit,
}

pub struct Root {
    flag: bool,

    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    first_configure: bool,
    width: u32,
    height: u32,
    shift: Option<u32>,
    layer: LayerSurface,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,

    widgets: Vec<Box<dyn Widget>>,
    processes: Vec<Box<dyn Process>>,
    env: Option<Rc<Environment>>,
}

impl CompositorHandler for Root {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        if let Err(a) = self.draw(qh) {
            println!("{a}");
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }
}

impl OutputHandler for Root {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for Root {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = NonZeroU32::new(configure.new_size.0).map_or(256, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(256, NonZeroU32::get);

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            if let Err(a) = self.draw(qh) {
                println!("{a}");
            }
        }
    }
}

impl SeatHandler for Root {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for Root {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _: Modifiers,
        _layout: u32,
    ) {
    }
}

impl PointerHandler for Root {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            match event.kind {
                Enter { .. } => {}
                Leave { .. } => {}
                Motion { .. } => {}
                Press { .. } => {
                    self.shift = self.shift.xor(Some(0));
                }
                Release { .. } => {}
                Axis { .. } => {}
            }
        }
    }
}

impl ShmHandler for Root {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for Root {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

impl Root {
    pub fn new(globals: &GlobalList, event_queue: &mut EventQueue<Root>) -> Result<Root> {
        let qh = event_queue.handle();

        let compositor =
            CompositorState::bind(globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(globals, &qh).expect("layer shell is not available");
        let shm = Shm::bind(globals, &qh).expect("wl_shm is not available");

        let surface = compositor.create_surface(&qh);

        let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("Bar"), None);

        let bar = Root {
            flag: true,

            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, &qh),
            output_state: OutputState::new(globals, &qh),
            shm,

            first_configure: true,
            width: 16,
            height: 16,
            shift: None,
            layer,
            keyboard: None,
            keyboard_focus: false,
            pointer: None,

            widgets: Vec::new(),
            processes: Vec::new(),
            env: None,
        };

        Ok(bar)
    }

    pub fn apply_config(&mut self, config: Config) -> Result<()> {
        let mut bar = Bar::new(None, config.bar.settings)?;

        for widget in config.bar.left {
            match widget {
                WidgetsList::Text(settings) => bar.create_child_left(Text::new, settings)?,
                WidgetsList::Clock(settings) => bar.create_child_left(Clock::new, settings)?,
                WidgetsList::Battery(settings) => bar.create_child_left(Battery::new, settings)?,
                WidgetsList::CPU(settings) => bar.create_child_left(CPU::new, settings)?,
                WidgetsList::Keyboard(wsettings, psettings) => {
                    self.create_process(clients::Keyboard::new, psettings)?;
                    bar.create_child_left(Keyboard::new, wsettings)?
                }
            }
        }

        for widget in config.bar.center {
            match widget {
                WidgetsList::Text(settings) => bar.create_child_center(Text::new, settings)?,
                WidgetsList::Clock(settings) => bar.create_child_center(Clock::new, settings)?,
                WidgetsList::Battery(settings) => {
                    bar.create_child_center(Battery::new, settings)?
                }
                WidgetsList::CPU(settings) => bar.create_child_center(CPU::new, settings)?,
                WidgetsList::Keyboard(wsettings, psettings) => {
                    self.create_process(clients::Keyboard::new, psettings)?;
                    bar.create_child_center(widgets::keyboard::Keyboard::new, wsettings)?
                }
            }
        }

        for widget in config.bar.right {
            match widget {
                WidgetsList::Text(settings) => bar.create_child_right(Text::new, settings)?,
                WidgetsList::Clock(settings) => bar.create_child_right(Clock::new, settings)?,
                WidgetsList::Battery(settings) => bar.create_child_right(Battery::new, settings)?,
                WidgetsList::CPU(settings) => bar.create_child_right(CPU::new, settings)?,
                WidgetsList::Keyboard(wsettings, psettings) => {
                    self.create_process(clients::Keyboard::new, psettings)?;
                    bar.create_child_right(widgets::keyboard::Keyboard::new, wsettings)?
                }
            }
        }

        self.add_widget(bar)?;
        Ok(())
    }

    pub fn init(&mut self, event_queue: &mut EventQueue<Root>) -> Result<&mut Self> {
        self.layer.set_anchor(Anchor::TOP);
        self.layer
            .set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        self.width = 1;
        self.height = 1;

        self.env = Some(Rc::new(Environment {
            config: Config::default(),
            drawer: RefCell::new(Drawer::new(&mut self.shm, 1, 1)),
            signals: RefCell::new(HashMap::new()),
        }));

        for process in &mut self.processes {
            process.bind(Rc::clone(self.env.as_ref().unwrap()))?;

            process.init()?;
        }

        for widget in &mut self.widgets {
            widget.bind(Rc::clone(self.env.as_ref().unwrap()))?;

            widget.init()?;
            let data = widget.data().borrow_mut();
            self.height = max(
                self.height,
                (data.height + data.position.1).try_into().unwrap(),
            );
        }

        event_queue.blocking_dispatch(self)?;

        for output in self.output_state().outputs() {
            let info = self
                .output_state
                .info(&output)
                .ok_or_else(|| "output has no info".to_owned())
                .unwrap();

            if let Some((width, height)) = info.logical_size {
                self.width = max(self.width, width as u32);
                self.height = min(self.height, height as u32);
            }
        }

        self.layer.set_size(self.width, self.height);
        self.layer.set_exclusive_zone(self.height as i32);
        self.layer.commit();

        self.env.as_ref().unwrap().drawer.borrow_mut().update_sizes(
            &mut self.shm,
            self.width as i32,
            self.height as i32,
        );

        Ok(self)
    }

    pub fn run(&mut self, event_queue: &mut EventQueue<Root>) -> Result<&mut Self> {
        let _ = event_queue.blocking_dispatch(self);

        loop {
            event_queue.blocking_dispatch(self)?;
            thread::sleep(Duration::from_millis(100));
        }

        //Ok(self)
    }

    pub fn add_font_by_name(&mut self, name: &'static str) -> Result<(), FontsError> {
        fonts::add_font_by_name(name)
    }

    pub fn add_widget<W>(&mut self, mut widget: W) -> Result<()>
    where
        W: Widget + 'static,
    {
        if let Some(env) = &self.env {
            widget.bind(Rc::clone(env))?;
        }
        self.widgets.push(Box::new(widget));
        Ok(())
    }

    pub fn create_widget<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: WidgetNew + Widget + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, WidgetError>,
    {
        self.widgets.push(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    pub fn create_process<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: ProcessNew + Process + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, ProcessError>,
    {
        self.processes
            .push(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) -> Result<()> {
        if self.env.is_none() {
            return Err(RootError::EnvironmentNotInit.into());
        }

        for process in &mut self.processes {
            process.run()?;
        }

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, self.width as i32, self.height as i32);

        for widget in self.widgets.iter() {
            widget.draw()?;
        }

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        self.env
            .as_ref()
            .unwrap()
            .drawer
            .borrow_mut()
            .commit(self.layer.wl_surface());

        self.flag = false;
        Ok(())
    }
}

delegate_compositor!(Root);
delegate_output!(Root);
delegate_shm!(Root);

delegate_seat!(Root);
delegate_keyboard!(Root);
delegate_pointer!(Root);

delegate_layer!(Root);

delegate_registry!(Root);
