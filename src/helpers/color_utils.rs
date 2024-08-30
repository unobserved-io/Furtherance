use iced::Color;
use palette::{FromColor, Srgb};
use rand::Rng;
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

impl FromHex for Color {
    fn from_hex(hex: &str) -> Result<Self, ParseIntError> {
        let hex = hex.trim_start_matches('#');

        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;

        Ok(Color::from_rgb(
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

impl ToHex for Color {
    fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }
}

pub trait RandomColor {
    fn random() -> Self;
}

impl RandomColor for Srgb {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Srgb::new(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        )
    }
}

impl RandomColor for Color {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Color::from_rgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        )
    }
}

pub trait ToIcedColor {
    fn to_iced_color(&self) -> Color;
}

impl ToIcedColor for Srgb {
    fn to_iced_color(&self) -> Color {
        let (r, g, b) = self.into_components();
        Color::from_rgb(r as f32, g as f32, b as f32)
    }
}

pub trait ToSrgb {
    fn to_srgb(&self) -> Srgb;
}

impl ToSrgb for Color {
    fn to_srgb(&self) -> Srgb {
        Srgb::new(self.r, self.g, self.b)
    }
}
