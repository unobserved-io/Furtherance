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

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FurShortcut {
    pub name: String,
    pub tags: String,
    pub project: String,
    pub rate: f32,
    pub currency: String,
    pub color_hex: String,
    pub uid: String,
    pub is_deleted: bool,
    pub last_updated: i64,
}

impl FurShortcut {
    pub fn new(
        name: String,
        tags: String,
        project: String,
        rate: f32,
        currency: String,
        color_hex: String,
    ) -> Self {
        let uid = generate_shortcut_uid(&name, &tags, &project, &rate, &currency);

        FurShortcut {
            name,
            tags,
            project,
            rate,
            currency,
            color_hex,
            uid,
            is_deleted: false,
            last_updated: Utc::now().timestamp(),
        }
    }
}

impl fmt::Display for FurShortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if !self.project.is_empty() {
            write!(f, " @{}", self.project)?;
        }

        if !self.tags.is_empty() {
            write!(f, " {}", self.tags)?;
        }

        if self.rate != 0.0 {
            write!(f, " ${:.2}", self.rate)?;
        }

        Ok(())
    }
}

pub fn generate_shortcut_uid(
    name: &str,
    tags: &str,
    project: &str,
    rate: &f32,
    currency: &str,
) -> String {
    let input = format!("{}{}{}{}{}", name, tags, project, rate, currency);

    blake3::hash(input.as_bytes()).to_hex().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedShortcut {
    pub encrypted_data: String,
    pub nonce: String,
    pub uid: String,
    pub last_updated: i64,
}
