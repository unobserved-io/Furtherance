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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FurView {
    Shortcuts,
    Timer,
    History,
    Report,
    Settings,
}

impl FurView {
    pub const ALL: [FurView; 5] = [
        FurView::Shortcuts,
        FurView::Timer,
        FurView::History,
        FurView::Report,
        FurView::Settings,
    ];
}

impl std::fmt::Display for FurView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FurView::Shortcuts => "Shortcuts",
                FurView::Timer => "Timer",
                FurView::History => "History",
                FurView::Report => "Report",
                FurView::Settings => "Settings",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FurAlert {
    DeleteGroupConfirmation,
    DeleteShortcutConfirmation,
    DeleteTaskConfirmation,
    Idle,
    PomodoroBreakOver,
    PomodoroOver,
    ShortcutExists,
}

#[derive(Debug)]
pub enum FurInspectorView {
    AddNewTask,
    AddShortcut,
    AddTaskToGroup,
    EditGroup,
    EditShortcut,
    EditTask,
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
        write!(
            f,
            "{}",
            match self {
                FurDateRange::PastWeek => "Past week",
                FurDateRange::ThirtyDays => "Past 30 days",
                FurDateRange::SixMonths => "Past 6 months",
                FurDateRange::AllTime => "All time",
                FurDateRange::Range => "Date range",
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
        write!(
            f,
            "{}",
            match self {
                FurTaskProperty::Title => "Title",
                FurTaskProperty::Project => "Project",
                FurTaskProperty::Tags => "Tags",
                FurTaskProperty::Rate => "Rate",
            }
        )
    }
}
