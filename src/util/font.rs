use std::rc::Rc;

use fontconfig::Fontconfig;
use thiserror::Error;

pub struct Fonts {
    fontconfig: Fontconfig,
    fonts: Rc<Vec<fontdue::Font>>,
}

#[derive(Error, Debug)]
pub enum FontsError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Font {0} was not found")]
    FontNotFound(&'static str),
}

impl Fonts {
    pub fn new() -> Option<Self> {
        let fc = Fontconfig::new()?;

        Some(Fonts {
            fontconfig: fc,
            fonts: Rc::new(Vec::new()),
        })
    }

    pub fn add_font_by_name(&mut self, name: &'static str) -> Result<(), FontsError> {
        let font = match self.fontconfig.find(name, None) {
            Some(f) => f,
            None => return Err(FontsError::FontNotFound(name)),
        };

        let bytes = match std::fs::read(font.path.as_path()) {
            Ok(b) => b,
            Err(e) => return Err(FontsError::IO(e)),
        };

        let font = fontdue::Font::from_bytes(
            bytes,
            fontdue::FontSettings {
                ..Default::default()
            },
        )
        .unwrap();

        let a = Rc::get_mut(&mut self.fonts).unwrap();
        a.push(font);

        Ok(())
    }

    pub fn fonts(&self) -> Rc<Vec<fontdue::Font>> {
        Rc::clone(&self.fonts)
    }
}
