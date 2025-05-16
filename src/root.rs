use std::{
    cmp::{max, min},
    error::Error,
    num::NonZeroU32,
};

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
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, EventQueue, QueueHandle,
};

use crate::widget::Widget;

pub struct Root {
    flag: bool,

    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    shift: Option<u32>,
    layer: LayerSurface,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,

    widgets: Vec<Box<dyn Widget>>,
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
        self.draw(qh);
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
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

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
            self.draw(qh);
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
        event: KeyEvent,
    ) {
        if event.keysym == Keysym::Escape {
            self.exit = true;
        }
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
    pub fn new(
        globals: &GlobalList,
        event_queue: &mut EventQueue<Root>,
    ) -> Result<Root, Box<dyn std::error::Error>> {
        let qh = event_queue.handle();

        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

        let surface = compositor.create_surface(&qh);

        let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("Bar"), None);

        let pool = SlotPool::new(16 * 16 * 4, &shm).expect("Failed to create pool");

        let bar = Root {
            flag: true,

            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),
            shm,

            exit: false,
            first_configure: true,
            pool,
            width: 16,
            height: 16,
            shift: None,
            layer,
            keyboard: None,
            keyboard_focus: false,
            pointer: None,

            widgets: Vec::new(),
        };

        Ok(bar)
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    pub fn init(
        &mut self,
        event_queue: &mut EventQueue<Root>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        self.layer.set_anchor(Anchor::TOP);
        self.layer
            .set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        self.width = 32;
        self.height = 60;

        event_queue.blocking_dispatch(self)?;

        for output in self.output_state().outputs() {
            let info = self
                .output_state
                .info(&output)
                .ok_or_else(|| "output has no info".to_owned())?;

            if let Some((width, height)) = info.logical_size {
                self.width = max(self.width, width as u32);
                self.height = min(self.height, height as u32);
            }
        }

        self.layer.set_size(self.width, self.height);
        self.layer.set_exclusive_zone(self.height as i32);
        self.layer.commit();

        let _ = self.pool.resize(
            ((self.width * self.height * 4) as usize)
                .try_into()
                .unwrap(),
        );

        Ok(self)
    }

    pub fn run(&mut self, event_queue: &mut EventQueue<Root>) -> Result<&mut Self, Box<dyn Error>> {
        let _ = event_queue.blocking_dispatch(self);

        loop {
            event_queue.blocking_dispatch(self)?;

            if self.exit {
                println!("exiting bar");
                break;
            }
        }

        Ok(self)
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width as usize;
        let height = self.height as usize;
        let stride = self.width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");
        {
            let _ = self.shift.unwrap_or(0);
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(_, chunk)| {
                    let a: u32 = 0xFF;
                    let r: u32 = 4;
                    let g: u32 = 165;
                    let b: u32 = 229;
                    let color = (a << 24) + (r << 16) + (g << 8) + b;

                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.to_le_bytes();
                });

            for (i, widget) in self.widgets.iter_mut().enumerate() {
                widget.draw(canvas, i * 300 + 33, 0, width);
            }
        }

        if let Some(shift) = &mut self.shift {
            *shift = (*shift + 1) % width as u32;
        }

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();

        self.flag = false;
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
