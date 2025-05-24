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
    FontNotFound(String),
}

impl Fonts {
    pub fn new() -> Option<Self> {
        let fc = match Fontconfig::new() {
            Some(f) => f,
            None => return None,
        };

        return Some(Fonts {
            fontconfig: fc,
            fonts: Rc::new(Vec::new()),
        });
    }

    pub fn add_font_by_name(&mut self, name: String) -> Result<(), FontsError> {
        let font = match self.fontconfig.find("Mono", None) {
            Some(f) => f,
            None => return Err(FontsError::FontNotFound(name)),
        };

        let bytes = match std::fs::read(&font.path.as_path()) {
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

        (*Rc::get_mut(&mut self.fonts).unwrap()).push(font);

        Ok(())
    }

    pub fn fonts(&self) -> Rc<Vec<fontdue::Font>> {
        Rc::clone(&self.fonts)
    }
}
