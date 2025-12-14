// Furtherance - Track your time without being tracked
// Copyright (C) 2025  Ricky Kresslein <rk@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use iced::Color;
use palette::Srgb;
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
        let mut rng = rand::rng();
        Srgb::new(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
        )
    }
}

impl RandomColor for Color {
    fn random() -> Self {
        let mut rng = rand::rng();
        Color::from_rgb(
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
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
