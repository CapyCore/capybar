#[derive(Clone, Copy)]
pub struct Color(u32);

impl Color {
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(u32::from_be_bytes([r, g, b, a]))
    }

    pub const fn to_rgba(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}
