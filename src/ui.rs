// Furtherance - Track your time without being tracked
// Copyright (C) 2022  Ricky Kresslein <rk@lakoliu.com>
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

pub mod window;
mod history_box;
mod tasks_page;
mod tasks_group;
mod task_row;
mod task_details;
mod preferences_window;

pub use window::FurtheranceWindow;
pub use history_box::FurHistoryBox;
pub use tasks_page::FurTasksPage;
pub use tasks_group::FurTasksGroup;
pub use task_row::FurTaskRow;
pub use task_details::FurTaskDetails;
pub use preferences_window::FurPreferencesWindow;
