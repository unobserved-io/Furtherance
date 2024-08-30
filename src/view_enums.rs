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
}
