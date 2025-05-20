#[derive(Clone, Copy)]
pub struct Color(u32);

impl Color {
    pub const NONE: Color = Color(0x00000000);
    pub const BLACK: Color = Color(0xFF000000);
    pub const WHITE: Color = Color(0xFFFFFFFF);
    pub const RED: Color = Color(0xFFFF0000);
    pub const GREEN: Color = Color(0xFF00FF00);
    pub const BLUE: Color = Color(0xFF0000FF);
    pub const CYAN: Color = Color(0xFF00FFFF);

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub const fn to_rgba(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}
