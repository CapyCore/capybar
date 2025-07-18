use core::fmt;
use std::error::Error;

use fontdue::{layout::GlyphPosition, Font};
use smithay_client_toolkit::shm::{
    slot::{Buffer, SlotPool},
    Shm,
};
use wayland_client::protocol::{wl_shm, wl_surface::WlSurface};

use crate::widgets::WidgetData;

use super::Color;

#[derive(Debug)]
pub enum DrawerError {
    OutOfBounds(usize, usize),
}

impl Error for DrawerError {}

impl fmt::Display for DrawerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfBounds(size, idx) => {
                write!(f, "Drawer out of bounds! Size is {size}, index is {idx}",)
            }
        }
    }
}

/// Utility structure used to simplify drawing the widgets.
#[derive(Debug)]
pub struct Drawer {
    pool: SlotPool,
    buffer: Option<Buffer>,

    width: i32,
    height: i32,
}

impl Drawer {
    pub fn new(shm: &mut Shm, width: i32, height: i32) -> Self {
        Drawer {
            pool: SlotPool::new((width * height * 4) as usize, shm).unwrap(),
            buffer: None,

            width,
            height,
        }
    }

    pub fn update_sizes(&mut self, shm: &mut Shm, width: i32, height: i32) {
        self.height = height;
        self.width = width;
        self.buffer = None;
        self.pool = SlotPool::new((width * height * 4) as usize, shm).unwrap();
    }

    /// Commit buffer to a surface
    pub fn commit(&self, surface: &WlSurface) {
        if let Some(buffer) = &self.buffer {
            buffer.attach_to(surface).expect("buffer attach");
            surface.commit();
        }
    }

    /// Put a single colored pixel in a relative space. Drawer converts local position in a widget
    /// to global buffer position using provided `WidgetData`.
    pub fn draw_pixel(&mut self, data: &WidgetData, pos: (usize, usize), color: Color) {
        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    self.width,
                    self.height,
                    self.width * 4,
                    wl_shm::Format::Argb8888,
                )
                .unwrap()
                .0
        });

        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        self.width,
                        self.height,
                        self.width * 4,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");
                *buffer = second_buffer;
                canvas
            }
        };

        let chunk_id = data.position.0 + pos.0 + (data.position.1 + pos.1) * self.width as usize;

        let chunk = canvas.chunks_exact_mut(4).nth(chunk_id);
        if let Some(chunk) = chunk {
            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            let c = Color::blend_colors(&Color::from_be_bytes(array), &color).to_be_bytes();
            *array = [c[2], c[1], c[0], c[3]];
        }
    }

    /// Draw a glyph from font. Drawer converts local position in a widget to global buf position
    /// using provided `WidgetData`.
    pub fn draw_glyph(
        &mut self,
        data: &WidgetData,
        glyph: &GlyphPosition,
        font: &Font,
        mut color: Color,
    ) {
        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    self.width,
                    self.height,
                    self.width * 4,
                    wl_shm::Format::Argb8888,
                )
                .unwrap()
                .0
        });

        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(
                        self.width,
                        self.height,
                        self.width * 4,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer");
                *buffer = second_buffer;
                canvas
            }
        };

        let bitmap = font
            .rasterize_indexed(glyph.key.glyph_index, glyph.key.px)
            .1;
        if glyph.char_data.is_whitespace() {
            return;
        }

        for x in 0..glyph.width {
            for y in 0..glyph.height {
                color.set_a(bitmap[x + y * glyph.width]);

                let chunk_id = data.position.0
                    + x
                    + glyph.x as usize
                    + (data.position.1 + y + glyph.y as usize) * self.width as usize;

                let chunk = canvas.chunks_exact_mut(4).nth(chunk_id);
                if let Some(chunk) = chunk {
                    let array: &mut [u8; 4] = chunk.try_into().unwrap();

                    *array =
                        Color::blend_colors(&Color::from_be_bytes(array), &color).to_be_bytes();
                }
            }
        }
    }
}
