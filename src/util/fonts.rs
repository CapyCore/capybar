use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex, MutexGuard},
};

use anyhow::Result;
use fontconfig::Fontconfig;
use thiserror::Error;

pub struct FontsMap {
    fontconfig: Fontconfig,
    fonts_map: Mutex<HashMap<String, usize>>,
    fonts_vec: Mutex<Vec<fontdue::Font>>,
}

static FONTS: LazyLock<FontsMap> = LazyLock::new(|| FontsMap::new().unwrap());

#[derive(Error, Debug)]
pub enum FontsError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Font {0} was not found")]
    FontNotFound(String),
}

impl FontsMap {
    fn new() -> Option<Self> {
        let fc = Fontconfig::new()?;

        Some(FontsMap {
            fontconfig: fc,
            fonts_map: Mutex::new(HashMap::new()),
            fonts_vec: Mutex::new(Vec::new()),
        })
    }
}

pub fn get() -> &'static LazyLock<FontsMap> {
    &FONTS
}

pub fn fonts_map() -> MutexGuard<'static, HashMap<String, usize>> {
    FONTS.fonts_map.lock().unwrap()
}

pub fn fonts_vec() -> MutexGuard<'static, Vec<fontdue::Font>> {
    FONTS.fonts_vec.lock().unwrap()
}

pub fn add_font_by_name(name: &str) -> Result<(), FontsError> {
    let font = match FONTS.fontconfig.find(name, None) {
        Some(f) => f,
        None => return Err(FontsError::FontNotFound(name.to_string())),
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

    let mut fonts_map = FONTS.fonts_map.lock().unwrap();
    let mut fonts_vec = FONTS.fonts_vec.lock().unwrap();
    fonts_map.insert(name.to_string(), fonts_vec.len());
    fonts_vec.push(font);

    Ok(())
}
