//! Colours written as `#rrggbb` or `#rrggbbaa` in theme packs.

use std::fmt;

use egui::Color32;
use serde::{Deserialize, Deserializer, de};

/// An sRGB colour with alpha, read from a hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    /// Parses `#rrggbb` or `#rrggbbaa`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let hex = text.strip_prefix('#').ok_or_else(|| format!("{text:?} must start with #"))?;
        if !(hex.len() == 6 || hex.len() == 8) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("{text:?} must be #rrggbb or #rrggbbaa"));
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("checked above");
        let a = if hex.len() == 8 { byte(6) } else { 255 };
        Ok(Self([byte(0), byte(2), byte(4), a]))
    }
}

impl From<Rgba> for Color32 {
    fn from(c: Rgba) -> Self {
        let [r, g, b, a] = c.0;
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}

impl fmt::Display for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b, a] = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

impl<'de> Deserialize<'de> for Rgba {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        Self::parse(&text).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_forms() {
        assert_eq!(Rgba::parse("#ff8000").unwrap(), Rgba([255, 128, 0, 255]));
        assert_eq!(Rgba::parse("#ff800080").unwrap(), Rgba([255, 128, 0, 128]));
        assert_eq!(Rgba::parse("#FF8000").unwrap().to_string(), "#ff8000ff");
    }

    #[test]
    fn refuses_garbage() {
        for bad in ["ff8000", "#ff80", "#gg8000", "#ff8000801", ""] {
            assert!(Rgba::parse(bad).is_err(), "{bad}");
        }
    }
}
