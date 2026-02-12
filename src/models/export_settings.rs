// Furtherance - Track your time without being tracked
// Copyright (C) 2026  Ricky Kresslein <r@kressle.in>
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

pub struct ExportSettings {
    pub name: bool,
    pub start_time: bool,
    pub stop_time: bool,
    pub tags: bool,
    pub project: bool,
    pub rate: bool,
    pub currency: bool,
    pub total_time: bool,
    pub total_earnings: bool,
}

impl ExportSettings {
    pub fn new() -> Self {
        Self {
            name: true,
            start_time: true,
            stop_time: true,
            tags: true,
            project: true,
            rate: true,
            currency: true,
            total_time: true,
            total_earnings: true,
        }
    }
}
