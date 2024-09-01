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
    DeleteTaskConfirmation,
    Idle,
    PomodoroBreakOver,
    PomodoroOver,
}

#[derive(Debug)]
pub enum FurInspectorView {
    AddNewTask,
    AddShortcut,
    AddTaskToGroup,
    EditTask,
    EditGroup,
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
    Charts,
    List,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    PomodoroOver,
    BreakOver,
    Idle,
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
