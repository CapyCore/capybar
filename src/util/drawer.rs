use core::fmt;
use std::error::Error;

use fontdue::{layout::GlyphPosition, Font};
use smithay_client_toolkit::shm::slot::{Buffer, SlotPool};
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
            Self::OutOfBounds(size, idx) => write!(
                f,
                "Drawer out of bounds! Size is {}, index is {}",
                size, idx
            ),
        }
    }
}

pub struct Drawer {
    pool: SlotPool,
    buffer: Option<Buffer>,

    width: i32,
    height: i32,
}

impl Drawer {
    pub fn new(mut pool: SlotPool, width: i32, height: i32) -> Self {
        let buffer = pool
            .create_buffer(width, height, width * 4, wl_shm::Format::Argb8888)
            .unwrap()
            .0;
        Drawer {
            pool,
            buffer: Some(buffer),

            width,
            height,
        }
    }

    pub fn commit(&self, surface: &WlSurface) {
        if let Some(buffer) = &self.buffer {
            buffer.attach_to(&surface).expect("buffer attach");
            surface.commit();
        }
    }

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

            *array = Color::blend_colors(&Color::from_be_bytes(array), &color).to_be_bytes();
        }
    }

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
