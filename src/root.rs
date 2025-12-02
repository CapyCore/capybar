use std::{
    cell::RefCell,
    cmp::{max, min},
    collections::HashMap,
    num::NonZeroU32,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use anyhow::{anyhow, Result};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::{PointerEvent, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use thiserror::Error;
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
    Connection, EventQueue, QueueHandle,
};

use crate::{
    config::Config,
    services::{Service, ServiceError, ServiceNew},
    util::{
        fonts::{self, FontsError},
        signals::{Signal, SignalNames},
        Drawer,
    },
    widgets::{
        containers::{bar::Bar, Container},
        Widget, WidgetNew,
    },
};

/// Structure containing things all the widgets in capybar needs access to
pub struct Environment {
    pub config: Config,
    pub drawer: RefCell<Drawer>,
    pub signals: RefCell<HashMap<SignalNames, Signal>>,
    pub mouse_position: RefCell<(f64, f64)>,
}

/// Structure describing all the capybar state that can be changed from outside
pub struct BarState {
    pub shutdown: Arc<AtomicBool>,
    pub shown: Arc<AtomicBool>,
}

impl Default for BarState {
    fn default() -> Self {
        Self {
            shutdown: Arc::new(AtomicBool::new(false)),
            shown: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[derive(Error, Debug)]
pub enum RootError {
    #[error("Environment is not initialised before drawing")]
    EnvironmentNotInit,
}

pub struct Root {
    state: BarState,

    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    first_configure: bool,
    width: u32,
    height: u32,
    layer: LayerSurface,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,

    bar: Option<Bar>,
    services: Vec<Box<dyn Service>>,
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
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
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
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
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
        use smithay_client_toolkit::seat::pointer::PointerEventKind;

        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }

            match &event.kind {
                PointerEventKind::Motion { .. } => {
                    if let Some(ref env) = self.env {
                        *env.mouse_position.borrow_mut() = event.position;
                    }
                }
                PointerEventKind::Leave { .. } => {
                    if let Some(ref env) = self.env {
                        *env.mouse_position.borrow_mut() = (-1.0, -1.0);
                    }
                }
                _ => {}
            }

            match self.bar.as_ref().unwrap().handle_mouse_event(event) {
                Ok(..) => {}
                Err(error) => {
                    println!("Mouse event failed with error:\n {error}");
                }
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
    pub fn new(
        globals: &GlobalList,
        event_queue: &mut EventQueue<Root>,
        bar: Option<Bar>,
    ) -> Result<Root> {
        let qh = event_queue.handle();

        let compositor =
            CompositorState::bind(globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(globals, &qh).expect("layer shell is not available");
        let shm = Shm::bind(globals, &qh).expect("wl_shm is not available");

        let surface = compositor.create_surface(&qh);

        let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("Bar"), None);

        let root = Root {
            state: BarState::default(),

            registry_state: RegistryState::new(globals),
            seat_state: SeatState::new(globals, &qh),
            output_state: OutputState::new(globals, &qh),
            shm,

            first_configure: true,
            width: 16,
            height: 16,
            layer,
            keyboard: None,
            keyboard_focus: false,
            pointer: None,

            bar,
            services: Vec::new(),
            env: None,
        };

        Ok(root)
    }

    pub fn apply_config(&mut self, config: Config) -> Result<()> {
        if self.bar.is_some() {
            return Err(anyhow!("Config can only be applied once"));
        }
        let mut bar = Bar::new(None, config.bar.settings)?;

        for widget in config.bar.left {
            widget.create_in_container(bar.left().get_mut())?;
        }

        for widget in config.bar.center {
            widget.create_in_container(bar.center().get_mut())?;
        }

        for widget in config.bar.right {
            widget.create_in_container(bar.right().get_mut())?;
        }

        self.bar = Some(bar);
        Ok(())
    }

    pub fn init(&mut self) -> Result<&mut Self> {
        if self.bar.is_none() {
            return Err(anyhow!("Empty bar can not be created"));
        }

        self.layer.set_anchor(Anchor::TOP);
        self.layer
            .set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        self.width = 1;
        self.height = 1;

        self.env = Some(Rc::new(Environment {
            config: Config::default(),
            drawer: RefCell::new(Drawer::new(&mut self.shm, 1, 1)),
            signals: RefCell::new(HashMap::new()),
            mouse_position: RefCell::new((-1.0, -1.0)),
        }));

        for service in &mut self.services {
            service.bind(Rc::clone(self.env.as_ref().unwrap()))?;

            service.init()?;
        }

        let bar = self.bar.as_mut().unwrap();
        bar.bind(Rc::clone(self.env.as_ref().unwrap()))?;
        bar.init()?;

        self.height = max(self.height, bar.data_mut().height as u32);

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

    /// Run the bar in a continious loop
    pub fn run_sync(&mut self, event_queue: &mut EventQueue<Root>) -> Result<&mut Self> {
        event_queue.blocking_dispatch(self)?;
        self.init()?;

        loop {
            thread::sleep(Duration::from_millis(10));
            event_queue.blocking_dispatch(self)?;
        }

        //Ok(self)
    }

    pub fn run_async(&mut self, event_queue: &mut EventQueue<Root>, state: BarState) -> Result<()> {
        event_queue.blocking_dispatch(self)?;
        self.init()?;
        self.state = state;

        while !self.state.shutdown.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
            event_queue.blocking_dispatch(self)?;
        }

        Ok(())
    }

    pub fn add_font_by_name(&mut self, name: &'static str) -> Result<(), FontsError> {
        fonts::add_font_by_name(name)
    }

    pub fn create_service<W, F>(&mut self, f: F, settings: W::Settings) -> Result<()>
    where
        W: ServiceNew + Service + 'static,
        F: FnOnce(Option<Rc<Environment>>, W::Settings) -> Result<W, ServiceError>,
    {
        self.services.push(Box::new(f(self.env.clone(), settings)?));
        Ok(())
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) -> Result<()> {
        if !self.state.shown.load(Ordering::SeqCst) {
            self.layer.set_exclusive_zone(0);

            self.layer
                .wl_surface()
                .damage_buffer(0, 0, self.width as i32, self.height as i32);

            self.layer
                .wl_surface()
                .frame(qh, self.layer.wl_surface().clone());

            let mut pool =
                SlotPool::new((self.width * self.height * 4) as usize, &self.shm).unwrap();
            let buf = pool
                .create_buffer(1, 1, 4, wayland_client::protocol::wl_shm::Format::Argb8888)
                .unwrap()
                .0;

            buf.attach_to(self.layer.wl_surface())
                .expect("buffer attach");
            self.layer.wl_surface().commit();

            return Ok(());
        }

        if self.env.is_none() {
            return Err(RootError::EnvironmentNotInit.into());
        }

        for service in &mut self.services {
            service.run()?;
        }

        self.bar.as_ref().unwrap().prepare()?;

        {
            let bar = self.bar.as_ref().unwrap().data();
            if self.width != bar.width as u32 || self.height != bar.height as u32 {
                self.width = bar.width as u32;
                self.height = bar.height as u32;

                self.layer.set_size(self.width, self.height);
                self.layer.set_exclusive_zone(self.height as i32);

                self.env.as_ref().unwrap().drawer.borrow_mut().update_sizes(
                    &mut self.shm,
                    self.width as i32,
                    self.height as i32,
                );
            }
        }

        self.layer.set_exclusive_zone(self.height as i32);

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, self.width as i32, self.height as i32);

        self.bar.as_ref().unwrap().run()?;
        self.bar.as_ref().unwrap().draw()?;

        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        self.env
            .as_ref()
            .unwrap()
            .drawer
            .borrow_mut()
            .commit(self.layer.wl_surface());

        Ok(())
    }

    pub fn bar(&self) -> &Option<Bar> {
        &self.bar
    }

    pub fn hide(&mut self) {
        self.state.shown.store(false, Ordering::SeqCst);
    }

    pub fn show(&mut self) {
        self.state.shown.store(true, Ordering::SeqCst);
    }

    pub fn toggle(&mut self) {
        self.state
            .shown
            .store(!self.state.shown.load(Ordering::SeqCst), Ordering::SeqCst);
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
