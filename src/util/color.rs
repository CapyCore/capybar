use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
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

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub const fn from_be_bytes(bytes: &[u8; 4]) -> Self {
        Self(u32::from_be_bytes(*bytes))
    }

    pub const fn to_be_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub fn set_a(&mut self, a: u8) {
        self.0 &= 0xFFFFFF00;
        self.0 |= a as u32;
    }

    pub fn blend_colors(background: &Color, foreground: &Color) -> Color {
        let bg = background.to_be_bytes();
        let fg = foreground.to_be_bytes();

        //TODO check if checking for a == 0 improves speed

        let bg = [
            bg[0] as f32 * bg[3] as f32,
            bg[1] as f32 * bg[3] as f32,
            bg[2] as f32 * bg[3] as f32,
            bg[3] as f32,
        ];
        let fg = [
            fg[0] as f32 * fg[3] as f32,
            fg[1] as f32 * fg[3] as f32,
            fg[2] as f32 * fg[3] as f32,
            fg[3] as f32,
        ];

        let coef = 1.0 - fg[3] / 255.0;
        let a = fg[3] + bg[3] * coef;
        Color::from_rgba(
            ((fg[0] + bg[0] * coef) / a).floor() as u8,
            ((fg[1] + bg[1] * coef) / a).floor() as u8,
            ((fg[2] + bg[2] * coef) / a).floor() as u8,
            (a * 255.0).floor() as u8,
        )
    }
}
