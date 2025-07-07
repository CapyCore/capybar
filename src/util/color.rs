use std::fmt::Display;

use serde::Deserialize;

/// Color structure used in capy. Color is stored as an rgba value.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
pub struct Color(u32);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:0>8x}", self.0)
    }
}

impl Color {
    pub const NONE: Color = Color(0x00000000);
    pub const BLACK: Color = Color(0x000000FF);
    pub const WHITE: Color = Color(0xFFFFFFFF);
    pub const RED: Color = Color(0xFF0000FF);
    pub const GREEN: Color = Color(0x00FF00FF);
    pub const BLUE: Color = Color(0x0000FFFF);
    pub const CYAN: Color = Color(0xFFFF00FF);
    pub const PINK: Color = Color(0xFF00FFFF);
    pub const YELLOW: Color = Color(0x00FFFFFF);

    pub const PURPLE: Color = Color(0x800080FF);

    pub const fn from_hex(hex: u32) -> Self {
        Self(hex)
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub const fn from_be_bytes(bytes: &[u8; 4]) -> Self {
        Self(u32::from_be_bytes(*bytes))
    }

    pub const fn from_le_bytes(bytes: &[u8; 4]) -> Self {
        Self(u32::from_le_bytes(*bytes))
    }

    pub fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Option<Color> {
        if !(0.0..=1.0).contains(&r)
            || !(0.0..=1.0).contains(&g)
            || !(0.0..=1.0).contains(&b)
            || !(0.0..=1.0).contains(&a)
        {
            return None;
        }

        Some(Self::from_rgba(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
            (a * 255.0).round() as u8,
        ))
    }

    pub const fn to_be_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub const fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn r(&self) -> u8 {
        ((self.0 & 0xFF000000) >> 24) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 & 0x00FF0000) >> 16) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.0 & 0x0000FF00) >> 8) as u8
    }

    pub fn a(&self) -> u8 {
        (self.0 & 0x000000FF) as u8
    }

    pub fn set_r(&mut self, a: u8) {
        self.0 &= 0x00FFFFFF;
        self.0 |= (a as u32) << 24;
    }

    pub fn set_g(&mut self, a: u8) {
        self.0 &= 0xFF00FFFF;
        self.0 |= (a as u32) << 16;
    }

    pub fn set_b(&mut self, a: u8) {
        self.0 &= 0xFFFF00FF;
        self.0 |= (a as u32) << 8;
    }

    pub fn set_a(&mut self, a: u8) {
        self.0 &= 0xFFFFFF00;
        self.0 |= a as u32;
    }

    pub fn blend_colors(background: &Color, foreground: &Color) -> Color {
        let bg = background.to_be_bytes();
        let fg = foreground.to_be_bytes();

        if fg[3] == 0 {
            return *background;
        }
        if fg[3] == 255 {
            return *foreground;
        }
        if bg[3] == 0 {
            return *foreground;
        }

        let bg_alpha = bg[3] as f32 / 255.0;
        let fg_alpha = fg[3] as f32 / 255.0;

        let a = fg_alpha + bg_alpha * (1.0 - fg_alpha);

        let blend_channel = |fg_c: u8, bg_c: u8| -> u8 {
            let fg_norm = fg_c as f32 / 255.0;
            let bg_norm = bg_c as f32 / 255.0;
            let blended = (fg_norm * fg_alpha) + (bg_norm * bg_alpha * (1.0 - fg_alpha));
            (blended / a * 255.0).round() as u8
        };

        Color::from_rgba(
            blend_channel(fg[0], bg[0]),
            blend_channel(fg[1], bg[1]),
            blend_channel(fg[2], bg[2]),
            (a * 255.0).floor() as u8,
        )
    }
}
