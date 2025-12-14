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

use crate::localization::Localization;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FurView {
    Shortcuts,
    Timer,
    Todo,
    Report,
    Settings,
}

impl FurView {
    pub const ALL: [FurView; 5] = [
        FurView::Shortcuts,
        FurView::Timer,
        FurView::Todo,
        FurView::Report,
        FurView::Settings,
    ];
}

impl std::fmt::Display for FurView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let localization = Localization::new();
        write!(
            f,
            "{}",
            match self {
                FurView::Shortcuts => localization.get_message("shortcuts", None),
                FurView::Timer => localization.get_message("timer", None),
                FurView::Todo => localization.get_message("todo", None),
                FurView::Report => localization.get_message("report", None),
                FurView::Settings => localization.get_message("settings", None),
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FurAlert {
    AutosaveRestored,
    DeleteEverythingConfirmation,
    DeleteGroupConfirmation,
    DeleteShortcutConfirmation,
    DeleteTaskConfirmation,
    DeleteTodoConfirmation,
    Idle,
    ImportMacDatabase,
    NotifyOfSync,
    PomodoroBreakOver,
    PomodoroOver,
    ShortcutExists,
}

#[derive(Debug)]
pub enum FurInspectorView {
    AddNewTask,
    AddNewTodo,
    AddShortcut,
    AddTaskToGroup,
    EditGroup,
    EditShortcut,
    EditTask,
    EditTodo,
}

#[derive(Debug, Clone)]
pub enum EditTaskProperty {
    Name,
    Tags,
    Project,
    Rate,
    StartTime,
    StopTime,
    StartDate,
    StopDate,
}

#[derive(Debug, Clone)]
pub enum EditTodoProperty {
    Task,
    Project,
    Tags,
    Rate,
    Date,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabId {
    General,
    Advanced,
    Pomodoro,
    Report,
    Data,
    Charts,
    List,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    PomodoroOver,
    BreakOver,
    Idle,
    Reminder,
}

#[derive(Debug, Clone)]
pub enum ChangeDB {
    Open,
    New,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FurDateRange {
    PastWeek,
    ThirtyDays,
    SixMonths,
    AllTime,
    Range,
}

impl FurDateRange {
    pub const ALL: [FurDateRange; 5] = [
        FurDateRange::PastWeek,
        FurDateRange::ThirtyDays,
        FurDateRange::SixMonths,
        FurDateRange::AllTime,
        FurDateRange::Range,
    ];
}

impl std::fmt::Display for FurDateRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let localization = Localization::new();
        write!(
            f,
            "{}",
            match self {
                FurDateRange::PastWeek => localization.get_message("past-week", None),
                FurDateRange::ThirtyDays => localization.get_message("past-thirty-days", None),
                FurDateRange::SixMonths => localization.get_message("past-six-months", None),
                FurDateRange::AllTime => localization.get_message("all-time", None),
                FurDateRange::Range => localization.get_message("date-range", None),
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FurTaskProperty {
    Title,
    Project,
    Tags,
    Rate,
}

impl FurTaskProperty {
    pub const ALL: [FurTaskProperty; 4] = [
        FurTaskProperty::Title,
        FurTaskProperty::Project,
        FurTaskProperty::Tags,
        FurTaskProperty::Rate,
    ];
}

impl std::fmt::Display for FurTaskProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let localization = Localization::new();
        write!(
            f,
            "{}",
            match self {
                FurTaskProperty::Title => localization.get_message("title", None),
                FurTaskProperty::Project => localization.get_message("project", None),
                FurTaskProperty::Tags => localization.get_message("tags", None),
                FurTaskProperty::Rate => localization.get_message("rate", None),
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerChoices {
    Official,
    Custom,
}

impl ServerChoices {
    pub const ALL: [ServerChoices; 2] = [ServerChoices::Official, ServerChoices::Custom];
}

impl std::fmt::Display for ServerChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let localization = Localization::new();
        write!(
            f,
            "{}",
            match self {
                ServerChoices::Official => localization.get_message("official-server", None),
                ServerChoices::Custom => localization.get_message("custom", None),
            }
        )
    }
}
