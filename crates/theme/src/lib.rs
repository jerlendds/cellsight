//! CellSight theme values and JSON loading.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, fs::File, io::Read, path::Path};

pub const PANEL: u32 = 0x161f20;
pub const BORDER: u32 = 0x162018;
pub const TEXT: u32 = 0xf0f5f1;
pub const MUTED: u32 = 0x909aa7;
pub const ACCENT: u32 = 0x3b6244;
pub const CANVAS: u32 = 0x0e0f0e;
pub const ACCENT_BTN: u32 = 0x1d2e25;
pub const BORDER_BTN: u32 = 0x233b2d;
pub const SURFACE_BTN: u32 = 0x181f1b;
pub const ACTIVE_BTN: u32 = 0x233b2d;

/// All colors used by the CellSight shell.
///
/// JSON colors may be `"#rrggbb"`, `"0xrrggbb"`, or integers. Serialization
/// always emits the more readable `"#rrggbb"` form.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    #[serde(with = "color")]
    pub panel: u32,
    #[serde(with = "color")]
    pub border: u32,
    #[serde(with = "color")]
    pub text: u32,
    #[serde(with = "color")]
    pub muted: u32,
    #[serde(with = "color")]
    pub accent: u32,
    #[serde(with = "color")]
    pub canvas: u32,
    #[serde(with = "color")]
    pub accent_btn: u32,
    #[serde(with = "color")]
    pub border_btn: u32,
    #[serde(with = "color")]
    pub surface_btn: u32,
    #[serde(with = "color")]
    pub active_btn: u32,
}

pub const DEFAULT: Theme = Theme {
    panel: PANEL,
    border: BORDER,
    text: TEXT,
    muted: MUTED,
    accent: ACCENT,
    canvas: CANVAS,
    accent_btn: ACCENT_BTN,
    border_btn: BORDER_BTN,
    surface_btn: SURFACE_BTN,
    active_btn: ACTIVE_BTN,
};

impl Default for Theme {
    fn default() -> Self {
        DEFAULT
    }
}

impl Theme {
    pub fn from_json_reader(reader: impl Read) -> Result<Self, Error> {
        serde_json::from_reader(reader).map_err(Error::Json)
    }

    pub fn from_json_str(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(Error::Json)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| Error::Io {
            path: path.to_owned(),
            source,
        })?;
        Self::from_json_reader(file)
    }

    pub fn color(&self, name: &str) -> Option<u32> {
        Some(match name {
            "panel" => self.panel,
            "border" => self.border,
            "text" => self.text,
            "muted" => self.muted,
            "accent" => self.accent,
            "canvas" => self.canvas,
            "accent_btn" => self.accent_btn,
            "border_btn" => self.border_btn,
            "surface_btn" => self.surface_btn,
            "active_btn" => self.active_btn,
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "could not read {}: {source}", path.display()),
            Self::Json(source) => write!(f, "invalid theme JSON: {source}"),
        }
    }
}

impl std::error::Error for Error {}

mod color {
    use super::*;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input {
        Number(u32),
        Text(String),
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
        let value = match Input::deserialize(deserializer)? {
            Input::Number(value) => value,
            Input::Text(value) => {
                let digits = value
                    .strip_prefix('#')
                    .or_else(|| value.strip_prefix("0x"))
                    .or_else(|| value.strip_prefix("0X"))
                    .unwrap_or(&value);
                if digits.len() != 6 {
                    return Err(serde::de::Error::custom(
                        "color must contain exactly 6 hex digits",
                    ));
                }
                u32::from_str_radix(digits, 16).map_err(serde::de::Error::custom)?
            }
        };
        if value > 0x00ff_ffff {
            return Err(serde::de::Error::custom("color must be a 24-bit RGB value"));
        }
        Ok(value)
    }

    pub fn serialize<S: Serializer>(value: &u32, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("#{value:06x}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_string_and_numeric_colors() {
        let json = r##"{
            "panel":"#161f20", "border":"0x162018", "text":15791601,
            "muted":"909aa7", "accent":"#3b6244", "canvas":"#0e0f0e",
            "accent_btn":"#1d2e25", "border_btn":"#233b2d",
            "surface_btn":"#181f1b", "active_btn":"#233b2d"
        }"##;
        let theme = Theme::from_json_str(json).unwrap();
        assert_eq!(theme, DEFAULT);
    }

    #[test]
    fn rejects_non_rgb_colors() {
        let json = serde_json::to_string(&DEFAULT)
            .unwrap()
            .replace("#161f20", "#ff161f20");
        assert!(Theme::from_json_str(&json).is_err());
    }
}
