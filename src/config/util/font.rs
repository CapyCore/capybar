use serde::{de::Visitor, Deserialize};

use crate::util::{fonts, Color};

#[derive(Default, Deserialize, Debug)]
pub struct PreloadedFonts {
    pub list: Vec<Font>,
}

impl PreloadedFonts {
    pub const fn default() -> Self {
        Self { list: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub name: String,
}

impl<'de> Deserialize<'de> for Font {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FontVisitor;

        impl<'de> Visitor<'de> for FontVisitor {
            type Value = Font;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expected font name that can be found using fontconfig")
            }

            fn visit_str<E>(self, name: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match fonts::add_font_by_name(name) {
                    Ok(_) => Ok(Font {
                        name: name.to_string(),
                    }),
                    Err(e) => Err(E::custom(e.to_string())),
                }
            }
        }
        deserializer.deserialize_str(FontVisitor)
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct FontStyle {
    pub name: String,
    #[serde(default = "FontStyle::default_text_size")]
    pub size: usize,
    #[serde(default = "FontStyle::default_text_color")]
    pub color: Color,
}

impl Font {
    pub const fn default() -> Self {
        Self {
            name: String::new(),
        }
    }
}

impl FontStyle {
    pub const fn default() -> Self {
        Self {
            name: String::new(),
            size: 0,
            color: Color::NONE,
        }
    }

    pub const fn default_text_color() -> Color {
        Color::BLACK
    }

    pub const fn default_text_size() -> usize {
        12
    }
}
