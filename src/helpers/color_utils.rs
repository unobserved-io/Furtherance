use palette::{FromColor, Srgb};
use std::num::ParseIntError;

pub trait FromHex {
    fn from_hex(hex: &str) -> Result<Self, ParseIntError>
    where
        Self: Sized;
}

impl FromHex for Srgb {
    fn from_hex(hex: &str) -> Result<Self, ParseIntError> {
        let hex = hex.trim_start_matches('#');

        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;

        Ok(Srgb::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        ))
    }
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for Srgb {
    fn to_hex(&self) -> String {
        let (r, g, b) = self.into_components();
        format!(
            "#{:02X}{:02X}{:02X}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }
}
