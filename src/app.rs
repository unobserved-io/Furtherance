// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
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

use core::f32;
use std::collections::BTreeMap;

use crate::{
    database::*,
    helpers::{
        color_utils::{FromHex, ToHex, ToSrgb},
        idle::get_idle_time,
    },
    models::{
        fur_idle::FurIdle, fur_pomodoro::FurPomodoro, fur_settings::FurSettings,
        fur_shortcut::FurShortcut, fur_task::FurTask, fur_task_group::FurTaskGroup,
        group_to_edit::GroupToEdit, shortcut_to_add::ShortcutToAdd, task_to_add::TaskToAdd,
        task_to_edit::TaskToEdit,
    },
    style,
    view_enums::*,
};
use chrono::{offset::LocalResult, DateTime, Datelike, Local, NaiveDate, NaiveTime};
use chrono::{Duration, TimeZone, Timelike};
use iced::widget::{toggler, Row};
use iced::Color;
use iced::{
    alignment, font,
    multi_window::Application,
    widget::{
        button, column, horizontal_space, pick_list, row, text, text_input, theme, vertical_space,
        Button, Column, Container, Scrollable,
    },
    window, Alignment, Command, Element, Length, Renderer, Theme,
};
use iced_aw::{
    color_picker,
    core::{
        color::HexString,
        icons::{bootstrap, Bootstrap, BOOTSTRAP_FONT_BYTES},
    },
    date_picker, modal, number_input, time_picker, Card, TabBarPosition, TabLabel, Tabs,
    TimePicker,
};
use palette::color_difference::Wcag21RelativeContrast;
use palette::Srgb;
use regex::Regex;
use tokio::time;

#[cfg(target_os = "linux")]
use crate::idle_wayland::run_on_idle;

pub struct Furtherance {
    current_view: FurView,
    displayed_alert: Option<FurAlert>,
    displayed_task_start_time: time_picker::Time,
    fur_settings: FurSettings,
    group_to_edit: Option<GroupToEdit>,
    idle: FurIdle,
    inspector_view: Option<FurInspectorView>,
    pomodoro: FurPomodoro,
    settings_active_tab: TabId,
    shortcuts: Vec<FurShortcut>,
    shortcut_to_add: Option<ShortcutToAdd>,
    show_timer_start_picker: bool,
    task_history: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>>,
    task_input: String,
    timer_is_running: bool,
    timer_start_time: DateTime<Local>,
    timer_stop_time: DateTime<Local>,
    timer_text: String,
    task_to_add: Option<TaskToAdd>,
    task_to_edit: Option<TaskToEdit>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddNewShortcutPressed,
    AddNewTaskPressed,
    AddTaskToGroup(GroupToEdit),
    AlertClose,
    CancelCurrentTaskStartTime,
    CancelGroupEdit,
    CancelShortcut,
    CancelShortcutColor,
    CancelTaskEdit,
    CancelTaskEditDateTime(EditTaskProperty),
    ChooseCurrentTaskStartTime,
    ChooseShortcutColor,
    ChooseTaskEditDateTime(EditTaskProperty),
    DeleteTasks,
    EditGroup(FurTaskGroup),
    EditShortcutTextChanged(String, EditTaskProperty),
    EditTask(FurTask),
    EditTaskTextChanged(String, EditTaskProperty),
    FontLoaded(Result<(), font::Error>),
    IdleDiscard,
    IdleReset,
    NavigateTo(FurView),
    PomodoroContinueAfterBreak,
    PomodoroSnooze,
    PomodoroStartBreak,
    PomodoroStop,
    PomodoroStopAfterBreak,
    RepeatLastTaskPressed(String),
    SaveGroupEdit,
    SaveShortcut,
    SaveTaskEdit,
    SettingsDefaultViewSelected(FurView),
    SettingsIdleTimeChanged(i64),
    SettingsIdleToggled(bool),
    SettingsPomodoroBreakLengthChanged(i64),
    SettingsPomodoroExtendedBreaksToggled(bool),
    SettingsPomodoroExtendedBreakIntervalChanged(u16),
    SettingsPomodoroExtendedBreakLengthChanged(i64),
    SettingsPomodoroLengthChanged(i64),
    SettingsPomodoroSnoozeLengthChanged(i64),
    SettingsPomodoroToggled(bool),
    SettingsTabSelected(TabId),
    ShortcutPressed(String),
    ShowAlert(FurAlert),
    StartStopPressed,
    StopwatchTick,
    SubmitCurrentTaskStartTime(time_picker::Time),
    SubmitShortcutColor(Color),
    SubmitTaskEditDate(date_picker::Date, EditTaskProperty),
    SubmitTaskEditTime(time_picker::Time, EditTaskProperty),
    TaskInputChanged(String),
    ToggleGroupEditor,
}

impl Application for Furtherance {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        // Load settings
        let settings = match FurSettings::new() {
            Ok(loaded_settings) => loaded_settings,
            Err(e) => {
                eprintln!("Error loading settings: {}", e);
                FurSettings::default()
            }
        };
        // Load or create database
        if let Err(e) = db_init() {
            eprintln!("Error loading database. Can't load or save data: {}", e);
        }
        // Update old furtherance databases with new properties
        if let Err(e) = db_upgrade_old() {
            eprintln!("Error upgrading legacy database: {}", e);
        }

        let mut furtherance = Furtherance {
            current_view: settings.default_view,
            displayed_alert: None,
            displayed_task_start_time: time_picker::Time::now_hm(true),
            fur_settings: settings,
            group_to_edit: None,
            idle: FurIdle::new(),
            pomodoro: FurPomodoro::new(),
            inspector_view: None,
            settings_active_tab: TabId::General,
            shortcuts: match db_retrieve_shortcuts() {
                Ok(shortcuts) => shortcuts,
                Err(e) => {
                    eprintln!("Error reading shortcuts from database: {}", e);
                    vec![]
                }
            },
            shortcut_to_add: None,
            show_timer_start_picker: false,
            task_history: get_task_history(),
            task_input: "".to_string(),
            timer_is_running: false,
            timer_start_time: Local::now(),
            timer_stop_time: Local::now(),
            timer_text: "0:00:00".to_string(),
            task_to_add: None,
            task_to_edit: None,
        };

        furtherance.timer_text = get_timer_text(&furtherance, 0);

        (
            furtherance,
            font::load(BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
        )
    }

    fn title(&self, _window_id: window::Id) -> String {
        "Furtherance".to_owned()
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        match dark_light::detect() {
            dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            dark_light::Mode::Dark => Theme::Dark,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::AddNewShortcutPressed => {
                self.shortcut_to_add = Some(ShortcutToAdd::new());
                self.inspector_view = Some(FurInspectorView::AddShortcut);
            }
            Message::AddNewTaskPressed => {
                self.task_to_add = Some(TaskToAdd::new());
                self.inspector_view = Some(FurInspectorView::AddNewTask);
            }
            Message::AddTaskToGroup(group_to_edit) => {
                self.task_to_add = Some(TaskToAdd::new_from(&group_to_edit));
                self.inspector_view = Some(FurInspectorView::AddTaskToGroup);
            }
            Message::AlertClose => self.displayed_alert = None,
            Message::CancelCurrentTaskStartTime => self.show_timer_start_picker = false,
            Message::CancelGroupEdit => {
                self.group_to_edit = None;
                self.inspector_view = None;
            }
            Message::CancelShortcut => {
                self.shortcut_to_add = None;
                self.inspector_view = None;
            }
            Message::CancelShortcutColor => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    shortcut_to_add.show_color_picker = false;
                }
            }
            Message::CancelTaskEdit => {
                self.task_to_edit = None;
                self.task_to_add = None;
                if self.group_to_edit.is_some() {
                    self.inspector_view = Some(FurInspectorView::EditGroup);
                } else {
                    self.inspector_view = None;
                }
            }
            Message::CancelTaskEditDateTime(property) => {
                if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                    match property {
                        EditTaskProperty::StartTime => {
                            task_to_edit.show_displayed_start_time_picker = false;
                        }
                        EditTaskProperty::StopTime => {
                            task_to_edit.show_displayed_stop_time_picker = false;
                        }
                        EditTaskProperty::StartDate => {
                            task_to_edit.show_displayed_start_date_picker = false;
                        }
                        EditTaskProperty::StopDate => {
                            task_to_edit.show_displayed_stop_date_picker = false;
                        }
                        _ => {}
                    }
                } else if let Some(task_to_add) = self.task_to_add.as_mut() {
                    match property {
                        EditTaskProperty::StartTime => {
                            task_to_add.show_start_time_picker = false;
                        }
                        EditTaskProperty::StopTime => {
                            task_to_add.show_stop_time_picker = false;
                        }
                        EditTaskProperty::StartDate => {
                            task_to_add.show_start_date_picker = false;
                        }
                        EditTaskProperty::StopDate => {
                            task_to_add.show_stop_date_picker = false;
                        }
                        _ => {}
                    }
                }
            }
            Message::ChooseCurrentTaskStartTime => self.show_timer_start_picker = true,
            Message::ChooseShortcutColor => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    shortcut_to_add.show_color_picker = true
                }
            }
            Message::ChooseTaskEditDateTime(property) => {
                if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                    match property {
                        EditTaskProperty::StartTime => {
                            task_to_edit.show_displayed_start_time_picker = true
                        }
                        EditTaskProperty::StopTime => {
                            task_to_edit.show_displayed_stop_time_picker = true
                        }
                        EditTaskProperty::StartDate => {
                            task_to_edit.show_displayed_start_date_picker = true;
                        }
                        EditTaskProperty::StopDate => {
                            task_to_edit.show_displayed_stop_date_picker = true;
                        }
                        _ => {}
                    }
                } else if let Some(task_to_add) = self.task_to_add.as_mut() {
                    match property {
                        EditTaskProperty::StartTime => {
                            task_to_add.show_start_time_picker = true;
                        }
                        EditTaskProperty::StopTime => {
                            task_to_add.show_stop_time_picker = true;
                        }
                        EditTaskProperty::StartDate => {
                            task_to_add.show_start_date_picker = true;
                        }
                        EditTaskProperty::StopDate => {
                            task_to_add.show_stop_date_picker = true;
                        }
                        _ => {}
                    }
                }
            }
            Message::DeleteTasks => {
                if let Some(task_to_edit) = &self.task_to_edit {
                    self.inspector_view = None;
                    let _ = db_delete_tasks_by_ids(vec![task_to_edit.id]);
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history();
                } else if let Some(group_to_edit) = &self.group_to_edit {
                    self.inspector_view = None;
                    let _ = db_delete_tasks_by_ids(group_to_edit.task_ids());
                    self.group_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history();
                }
            }
            Message::EditGroup(task_group) => {
                if task_group.tasks.len() == 1 {
                    if let Some(task_to_edit) = task_group.tasks.first() {
                        self.task_to_edit = Some(TaskToEdit::new_from(task_to_edit));
                    }
                    self.inspector_view = Some(FurInspectorView::EditTask);
                } else {
                    self.group_to_edit = Some(GroupToEdit::new_from(&task_group));
                    self.inspector_view = Some(FurInspectorView::EditGroup);
                }
            }
            Message::EditShortcutTextChanged(new_value, property) => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    match property {
                        EditTaskProperty::Name => {
                            if new_value.contains('#')
                                || new_value.contains('@')
                                || new_value.contains('$')
                            {
                                shortcut_to_add.invalid_input_error_message =
                                    "Task name cannot contain #, @, or $.".to_string();
                            } else {
                                shortcut_to_add.name = new_value;
                                shortcut_to_add.invalid_input_error_message = String::new();
                            }
                        }
                        EditTaskProperty::Project => {
                            if new_value.contains('#')
                                || new_value.contains('@')
                                || new_value.contains('$')
                            {
                                // TODO: Change to .input_error system
                                shortcut_to_add.input_error("Project cannot contain #, @, or $.");
                            } else {
                                shortcut_to_add.project = new_value;
                            }
                        }
                        EditTaskProperty::Tags => {
                            if new_value.contains('@') || new_value.contains('$') {
                                shortcut_to_add.input_error("Tags cannot contain @ or $.");
                            } else if !new_value.is_empty() && new_value.chars().next() != Some('#')
                            {
                                shortcut_to_add.input_error("Tags must start with a #.");
                            } else {
                                shortcut_to_add.tags = new_value;
                                shortcut_to_add.input_error("");
                            }
                        }
                        EditTaskProperty::Rate => {
                            let new_value_parsed = new_value.parse::<f32>();
                            if new_value.is_empty() {
                                shortcut_to_add.new_rate = String::new();
                            } else if new_value.contains('$') {
                                shortcut_to_add.input_error("Do not include a $ in the rate.");
                            } else if new_value_parsed.is_ok()
                                && has_max_two_decimals(&new_value)
                                && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                            {
                                shortcut_to_add.new_rate = new_value;
                                shortcut_to_add.input_error("");
                            } else {
                                shortcut_to_add.input_error("Rate must be a valid dollar amount.");
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::EditTask(task) => {
                self.task_to_edit = Some(TaskToEdit::new_from(&task));
                self.inspector_view = Some(FurInspectorView::EditTask);
            }
            Message::EditTaskTextChanged(new_value, property) => {
                match self.inspector_view {
                    Some(FurInspectorView::AddNewTask) => {
                        if let Some(task_to_add) = self.task_to_add.as_mut() {
                            match property {
                                EditTaskProperty::Name => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        task_to_add.invalid_input_error_message =
                                            "Task name cannot contain #, @, or $.".to_string();
                                    } else {
                                        task_to_add.name = new_value;
                                        task_to_add.invalid_input_error_message = String::new();
                                    }
                                }
                                EditTaskProperty::Project => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        // TODO: Change to .input_error system
                                        task_to_add
                                            .input_error("Project cannot contain #, @, or $.");
                                    } else {
                                        task_to_add.project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        task_to_add.input_error("Tags cannot contain @ or $.");
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        task_to_add.input_error("Tags must start with a #.");
                                    } else {
                                        task_to_add.tags = new_value;
                                        task_to_add.input_error("");
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        task_to_add.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        task_to_add.input_error("Do not include a $ in the rate.");
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(&new_value)
                                        && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                    {
                                        task_to_add.new_rate = new_value;
                                        task_to_add.input_error("");
                                    } else {
                                        task_to_add
                                            .input_error("Rate must be a valid dollar amount.");
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(FurInspectorView::EditTask) => {
                        if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                            match property {
                                EditTaskProperty::Name => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        task_to_edit
                                            .input_error("Task name cannot contain #, @, or $.");
                                    } else {
                                        task_to_edit.new_name = new_value;
                                        task_to_edit.input_error("");
                                    }
                                }
                                EditTaskProperty::Project => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        task_to_edit
                                            .input_error("Project cannot contain #, @, or $.");
                                    } else {
                                        task_to_edit.new_project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        task_to_edit.input_error("Tags cannot contain @ or $.");
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        task_to_edit.input_error("Tags must start with a #.");
                                    } else {
                                        task_to_edit.new_tags = new_value;
                                        task_to_edit.input_error("");
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        task_to_edit.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        task_to_edit.input_error("Do not include a $ in the rate.");
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(&new_value)
                                        && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                    {
                                        task_to_edit.new_rate = new_value;
                                        task_to_edit.input_error("");
                                    } else {
                                        task_to_edit
                                            .input_error("Rate must be a valid dollar amount.");
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(FurInspectorView::EditGroup) => {
                        if let Some(group_to_edit) = self.group_to_edit.as_mut() {
                            match property {
                                EditTaskProperty::Name => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        group_to_edit
                                            .input_error("Task name cannot contain #, @, or $.");
                                    } else {
                                        group_to_edit.new_name = new_value;
                                        group_to_edit.input_error("");
                                    }
                                }
                                EditTaskProperty::Project => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        group_to_edit
                                            .input_error("Project cannot contain #, @, or $.");
                                    } else {
                                        group_to_edit.new_project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        group_to_edit.input_error("Tags cannot contain @ or $.");
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        group_to_edit.input_error("Tags must start with a #.");
                                    } else {
                                        group_to_edit.new_tags = new_value;
                                        group_to_edit.input_error("");
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        group_to_edit.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        group_to_edit
                                            .input_error("Do not include a $ in the rate.");
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(&new_value)
                                        && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                    {
                                        group_to_edit.new_rate = new_value;
                                        group_to_edit.input_error("");
                                    } else {
                                        group_to_edit
                                            .input_error("Rate must be a valid dollar amount.");
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::FontLoaded(_) => {}
            Message::IdleDiscard => {
                stop_timer(self, self.idle.start_time);
                self.displayed_alert = None;
            }
            Message::IdleReset => {
                self.idle = FurIdle::new();
                self.displayed_alert = None;
                // TODO: Remove pending notifications?
            }
            Message::NavigateTo(destination) => {
                if self.current_view != destination {
                    self.inspector_view = None;
                    self.current_view = destination;
                }
            }
            Message::PomodoroContinueAfterBreak => {
                self.timer_is_running = false;
                let original_task_input = self.task_input.clone();
                self.pomodoro.on_break = false;
                self.pomodoro.snoozed = false;
                reset_timer(self);
                self.task_input = original_task_input;
                self.displayed_alert = None;
                start_timer(self);
                return Command::perform(get_timer_duration(), |_| Message::StopwatchTick);
            }
            Message::PomodoroSnooze => {
                self.pomodoro.snoozed = true;
                self.pomodoro.snoozed_at = Local::now();
                // Timer is still running but we want to first show the snooze time total
                self.timer_text = get_stopped_timer_text(self);
                self.displayed_alert = None;
                return Command::perform(get_timer_duration(), |_| Message::StopwatchTick);
            }
            Message::PomodoroStartBreak => {
                let original_task_input = self.task_input.clone();
                let pomodoro_stop_time = self.timer_start_time
                    + Duration::minutes(self.fur_settings.pomodoro_break_length);
                self.pomodoro.on_break = true;
                self.pomodoro.snoozed = false;
                stop_timer(self, pomodoro_stop_time);
                self.task_input = original_task_input;
                self.displayed_alert = None;
                start_timer(self);
                return Command::perform(get_timer_duration(), |_| Message::StopwatchTick);
            }
            Message::PomodoroStop => {
                self.pomodoro.snoozed = false;
                stop_timer(self, Local::now());
                self.displayed_alert = None;
                self.pomodoro.sessions = 0;
            }
            Message::PomodoroStopAfterBreak => {
                self.timer_is_running = false;
                self.pomodoro.on_break = false;
                self.pomodoro.snoozed = false;
                reset_timer(self);
                self.pomodoro.sessions = 0;
                self.displayed_alert = None;
            }
            Message::RepeatLastTaskPressed(last_task_input) => {
                self.task_input = last_task_input;
                self.current_view = FurView::Timer;
                return Command::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::SaveGroupEdit => {
                if let Some(group_to_edit) = &self.group_to_edit {
                    let _ = db_update_group_of_tasks(group_to_edit);
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history();
                }
            }
            Message::SaveShortcut => {
                if let Some(shortcut_to_add) = &self.shortcut_to_add {
                    if let Err(e) = db_write_shortcut(FurShortcut {
                        id: 0,
                        name: shortcut_to_add.name.clone(),
                        tags: shortcut_to_add.tags.clone(),
                        project: shortcut_to_add.project.trim().to_string(),
                        rate: shortcut_to_add
                            .new_rate
                            .trim()
                            .parse::<f32>()
                            .unwrap_or(0.0),
                        currency: String::new(),
                        color_hex: shortcut_to_add.color.to_hex(),
                    }) {
                        eprintln!("Failed to write shortcut to database: {}", e);
                    }
                    self.inspector_view = None;
                    self.shortcut_to_add = None;
                    if let Ok(all_shortcuts) = db_retrieve_shortcuts() {
                        self.shortcuts = all_shortcuts;
                    };
                }
            }
            Message::SaveTaskEdit => {
                if let Some(task_to_edit) = &self.task_to_edit {
                    let tags_without_first_pound = task_to_edit
                        .new_tags
                        .trim()
                        .strip_prefix('#')
                        .unwrap_or(&task_to_edit.new_tags)
                        .trim()
                        .to_string();
                    let _ = db_update_task(FurTask {
                        id: task_to_edit.id,
                        name: task_to_edit.new_name.trim().to_string(),
                        start_time: task_to_edit.new_start_time,
                        stop_time: task_to_edit.new_stop_time,
                        tags: tags_without_first_pound,
                        project: task_to_edit.new_project.trim().to_string(),
                        rate: task_to_edit.new_rate.trim().parse::<f32>().unwrap_or(0.0),
                        currency: String::new(),
                    });
                    self.inspector_view = None;
                    self.task_to_edit = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history();
                } else if let Some(task_to_add) = &self.task_to_add {
                    let tags_without_first_pound = task_to_add
                        .tags
                        .trim()
                        .strip_prefix('#')
                        .unwrap_or(&task_to_add.tags)
                        .trim()
                        .to_string();
                    let _ = db_write_task(FurTask {
                        id: 0,
                        name: task_to_add.name.trim().to_string(),
                        start_time: task_to_add.start_time,
                        stop_time: task_to_add.stop_time,
                        tags: tags_without_first_pound,
                        project: task_to_add.project.trim().to_string(),
                        rate: task_to_add.new_rate.trim().parse::<f32>().unwrap_or(0.0),
                        currency: String::new(),
                    });
                    self.inspector_view = None;
                    self.task_to_add = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history();
                }
            }
            Message::SettingsDefaultViewSelected(selected_view) => {
                if let Err(e) = self.fur_settings.change_default_view(&selected_view) {
                    eprintln!("Failed to change default_view in settings: {}", e);
                }
            }
            Message::SettingsIdleTimeChanged(new_minutes) => {
                if new_minutes >= 1 {
                    if let Err(e) = self.fur_settings.change_chosen_idle_time(&new_minutes) {
                        eprintln!("Failed to change chosen_idle_time in settings: {}", e);
                    }
                }
            }
            Message::SettingsIdleToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_notify_on_idle(&new_value) {
                    eprintln!("Failed to change notify_on_idle in settings: {}", e);
                }
            }
            Message::SettingsPomodoroBreakLengthChanged(new_minutes) => {
                if new_minutes >= 1 {
                    if let Err(e) = self.fur_settings.change_pomodoro_break_length(&new_minutes) {
                        eprintln!("Failed to change pomodoro_break_length in settings: {}", e);
                    }
                }
            }
            Message::SettingsPomodoroExtendedBreaksToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_pomodoro_extended_breaks(&new_value)
                {
                    eprintln!(
                        "Failed to change pomdoro_extended_breaks in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsPomodoroExtendedBreakIntervalChanged(new_interval) => {
                if new_interval >= 1 {
                    if let Err(e) = self
                        .fur_settings
                        .change_pomodoro_extended_break_interval(&new_interval)
                    {
                        eprintln!(
                            "Failed to change pomdoro_extended_break_interval in settings: {}",
                            e
                        );
                    }
                }
            }
            Message::SettingsPomodoroExtendedBreakLengthChanged(new_minutes) => {
                if new_minutes >= 1 {
                    if let Err(e) = self
                        .fur_settings
                        .change_pomodoro_extended_break_length(&new_minutes)
                    {
                        eprintln!(
                            "Failed to change pomdoro_extended_break_length in settings: {}",
                            e
                        );
                    }
                }
            }
            Message::SettingsPomodoroLengthChanged(new_minutes) => {
                if new_minutes >= 1 {
                    if let Err(e) = self.fur_settings.change_pomodoro_length(&new_minutes) {
                        eprintln!("Failed to change pomodoro_length in settings: {}", e);
                    }
                    self.timer_text = get_timer_text(
                        &self,
                        Local::now()
                            .signed_duration_since(self.timer_start_time)
                            .num_seconds(),
                    );
                }
            }
            Message::SettingsPomodoroSnoozeLengthChanged(new_minutes) => {
                if new_minutes >= 1 {
                    if let Err(e) = self
                        .fur_settings
                        .change_pomodoro_snooze_length(&new_minutes)
                    {
                        eprintln!("Failed to change pomodoro_snooze_length in settings: {}", e);
                    }
                }
            }
            Message::SettingsPomodoroToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_pomodoro(&new_value) {
                    eprintln!("Failed to change pomodoro in settings: {}", e);
                }
                self.timer_text = get_timer_text(
                    &self,
                    Local::now()
                        .signed_duration_since(self.timer_start_time)
                        .num_seconds(),
                );
            }
            Message::SettingsTabSelected(new_tab) => self.settings_active_tab = new_tab,
            Message::ShortcutPressed(shortcut_task_input) => {
                self.task_input = shortcut_task_input;
                self.current_view = FurView::Timer;
                return Command::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::ShowAlert(alert_to_show) => self.displayed_alert = Some(alert_to_show),
            Message::StartStopPressed => {
                if self.timer_is_running {
                    // Do not more declarations to after if else
                    // They are needed in this position to properly initiate timer on reset
                    if self.pomodoro.on_break {
                        self.timer_is_running = false;
                        self.pomodoro.on_break = false;
                        self.pomodoro.snoozed = false;
                        self.pomodoro.sessions = 0;
                        reset_timer(self);
                    } else {
                        self.pomodoro.on_break = false;
                        self.pomodoro.snoozed = false;
                        self.pomodoro.sessions = 0;
                        stop_timer(self, Local::now());
                    }
                    return Command::none();
                } else {
                    start_timer(self);
                    return Command::perform(get_timer_duration(), |_| Message::StopwatchTick);
                }
            }
            Message::StopwatchTick => {
                if self.timer_is_running {
                    let duration = Local::now().signed_duration_since(self.timer_start_time);
                    let seconds_elapsed = duration.num_seconds();
                    self.timer_text = get_timer_text(self, seconds_elapsed);
                    if self.fur_settings.pomodoro
                        && self.timer_text == "0:00:00".to_string()
                        && seconds_elapsed > 2
                    {
                        // Check if idle or other alert is being displayed so as not to replace it
                        if self.displayed_alert.is_none() {
                            if self.pomodoro.on_break {
                                self.displayed_alert = Some(FurAlert::PomodoroBreakOver);
                            } else {
                                self.displayed_alert = Some(FurAlert::PomodoroOver);
                            }
                        }
                        return Command::none();
                    }

                    if self.fur_settings.notify_on_idle
                        && self.displayed_alert != Some(FurAlert::PomodoroOver)
                    {
                        let idle_time = get_idle_time() as i64;
                        if idle_time >= self.fur_settings.chosen_idle_time * 60
                            && !self.idle.reached
                        {
                            // User is idle
                            self.idle.reached = true;
                            self.idle.start_time = Local::now()
                                - Duration::seconds(self.fur_settings.chosen_idle_time * 60);
                        } else if idle_time < self.fur_settings.chosen_idle_time * 60
                            && self.idle.reached
                            && !self.idle.notified
                        {
                            // User is back - show idle message
                            self.idle.notified = true;
                            // TODO: Set up notification to display
                            self.displayed_alert = Some(FurAlert::Idle);
                        }
                    }

                    return Command::perform(get_timer_duration(), |_| Message::StopwatchTick);
                } else {
                    return Command::none();
                }
            }
            Message::SubmitCurrentTaskStartTime(new_time) => {
                match convert_iced_time_to_chrono_local(new_time) {
                    LocalResult::Single(local_time) => {
                        if local_time <= Local::now() {
                            self.displayed_task_start_time = new_time;
                            self.timer_start_time = local_time;
                            self.show_timer_start_picker = false;
                        }
                    }
                    _ => {
                        self.show_timer_start_picker = false;
                        eprintln!("Error converting chosen time to local time.");
                    }
                }
            }
            Message::SubmitShortcutColor(new_color) => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    shortcut_to_add.color = new_color;
                    shortcut_to_add.show_color_picker = false;
                }
            }
            Message::SubmitTaskEditDate(new_date, property) => {
                if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                    match property {
                        EditTaskProperty::StartDate => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_date_with_time(task_to_edit.new_start_time, new_date)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time < task_to_edit.new_stop_time
                                {
                                    task_to_edit.displayed_start_date = new_date;
                                    task_to_edit.new_start_time = new_local_date_time;
                                    task_to_edit.show_displayed_start_date_picker = false;
                                }
                            }
                        }
                        EditTaskProperty::StopDate => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_date_with_time(task_to_edit.new_stop_time, new_date)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time > task_to_edit.new_start_time
                                {
                                    task_to_edit.displayed_stop_date = new_date;
                                    task_to_edit.new_stop_time = new_local_date_time;
                                    task_to_edit.show_displayed_stop_date_picker = false;
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Some(task_to_add) = self.task_to_add.as_mut() {
                    match property {
                        EditTaskProperty::StartDate => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_date_with_time(task_to_add.start_time, new_date)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time < task_to_add.stop_time
                                {
                                    task_to_add.displayed_start_date = new_date;
                                    task_to_add.start_time = new_local_date_time;
                                    task_to_add.show_start_date_picker = false;
                                }
                            }
                        }
                        EditTaskProperty::StopDate => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_date_with_time(task_to_add.stop_time, new_date)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time > task_to_add.start_time
                                {
                                    task_to_add.displayed_stop_date = new_date;
                                    task_to_add.stop_time = new_local_date_time;
                                    task_to_add.show_stop_date_picker = false;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::SubmitTaskEditTime(new_time, property) => {
                // TODO: Edit to fix issues in greater than stop, etc. like below
                if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                    match property {
                        EditTaskProperty::StartTime => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_time_with_date(task_to_edit.new_start_time, new_time)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time < task_to_edit.new_stop_time
                                {
                                    task_to_edit.displayed_start_time = new_time;
                                    task_to_edit.new_start_time = new_local_date_time;
                                    task_to_edit.show_displayed_start_time_picker = false;
                                }
                            }
                        }
                        EditTaskProperty::StopTime => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_time_with_date(task_to_edit.new_stop_time, new_time)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time > task_to_edit.new_start_time
                                {
                                    task_to_edit.displayed_stop_time = new_time;
                                    task_to_edit.new_stop_time = new_local_date_time;
                                    task_to_edit.show_displayed_stop_time_picker = false;
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Some(task_to_add) = self.task_to_add.as_mut() {
                    match property {
                        EditTaskProperty::StartTime => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_time_with_date(task_to_add.start_time, new_time)
                            {
                                if new_local_date_time <= Local::now()
                                    && new_local_date_time < task_to_add.stop_time
                                {
                                    task_to_add.displayed_start_time = new_time;
                                    task_to_add.start_time = new_local_date_time;
                                    task_to_add.show_start_time_picker = false;
                                }
                            }
                        }
                        EditTaskProperty::StopTime => {
                            if let LocalResult::Single(new_local_date_time) =
                                combine_chosen_time_with_date(task_to_add.stop_time, new_time)
                            {
                                if new_local_date_time > task_to_add.start_time {
                                    task_to_add.displayed_stop_time = new_time;
                                    task_to_add.stop_time = new_local_date_time;
                                    task_to_add.show_stop_time_picker = false;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::TaskInputChanged(new_value) => {
                // Handle all possible task input checks here rather than on start/stop press
                // If timer is running, task can never be empty
                if self.timer_is_running {
                    if new_value.trim().is_empty() {
                        return Command::none();
                    }
                }
                let new_value_trimmed = new_value.trim_start();
                // Doesn't start with @
                if new_value_trimmed.chars().next() != Some('@')
                    // Doesn't start with #
                    && new_value_trimmed.chars().next() != Some('#')
                    // Doesn't start with $
                    && new_value_trimmed.chars().next() != Some('$')
                    // No more than 1 @
                    && new_value_trimmed.chars().filter(|&c| c == '@').count() < 2
                    // No more than 1 $
                    && new_value_trimmed.chars().filter(|&c| c == '$').count() < 2
                {
                    // Check if there is a $ and the subsequent part is a parseable f32
                    if let Some(dollar_index) = new_value_trimmed.find('$') {
                        let after_dollar = &new_value_trimmed[dollar_index + 1..];
                        if after_dollar.is_empty() {
                            // Allow typing the $ in the first place
                            self.task_input = new_value_trimmed.to_string();
                        } else {
                            // Find the parseable number right after the $
                            let end_index = after_dollar.find(' ').unwrap_or(after_dollar.len());
                            let number_str = &after_dollar[..end_index];
                            let parsed_num = number_str.parse::<f32>();

                            if parsed_num.is_ok()
                                && has_max_two_decimals(&number_str)
                                && parsed_num.unwrap_or(f32::MAX) < f32::MAX
                            {
                                let remaining_str = &after_dollar[end_index..].trim_start();
                                if remaining_str.is_empty() {
                                    // Allow a number to be typed after the $
                                    self.task_input = new_value_trimmed.to_string();
                                } else {
                                    // Only allow a space, @, or # to be typed after the $ amount
                                    if remaining_str.starts_with('@')
                                        || remaining_str.starts_with('#')
                                    {
                                        self.task_input = new_value_trimmed.to_string();
                                    }
                                }
                            }
                        }
                    } else {
                        // If there is no $, no other checks are necessary
                        self.task_input = new_value_trimmed.to_string();
                    }
                }
            }
            Message::ToggleGroupEditor => {
                self.group_to_edit
                    .as_mut()
                    .map(|group| group.is_in_edit_mode = !group.is_in_edit_mode);
            }
        }
        Command::none()
    }

    fn view(&self, _window_id: window::Id) -> Element<Message> {
        // MARK: SIDEBAR
        let sidebar = Container::new(
            column![
                nav_button("Shortcuts", FurView::Shortcuts),
                nav_button("Timer", FurView::Timer),
                nav_button("History", FurView::History),
                nav_button("Report", FurView::Report),
                vertical_space().height(Length::Fill),
                if self.timer_is_running && self.current_view != FurView::Timer {
                    text(convert_timer_text_to_vertical_hms(&self.timer_text))
                        .size(50)
                        .style(if self.pomodoro.on_break {
                            theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0))
                        } else {
                            theme::Text::Default
                        })
                } else {
                    text("")
                },
                nav_button("Settings", FurView::Settings)
            ]
            .spacing(12)
            .align_items(Alignment::Start),
        )
        .width(175)
        .padding(10)
        .clip(true)
        .style(style::gray_background);

        // MARK: Shortcuts
        let mut shortcuts_column = column![].padding(20);
        for shortcut in &self.shortcuts {
            shortcuts_column =
                shortcuts_column.push(shortcut_button(self.timer_is_running, shortcut));
        }
        let shortcuts_view = column![
            row![
                horizontal_space(),
                button(bootstrap::icon_to_text(bootstrap::Bootstrap::PlusLg))
                    .on_press(Message::AddNewShortcutPressed)
                    .style(theme::Button::Text),
            ]
            .padding([10, 20]),
            Scrollable::new(shortcuts_column,)
        ];

        // MARK: TIMER
        let timer_view = column![
            row![
                button(bootstrap::icon_to_text(bootstrap::Bootstrap::ArrowRepeat))
                    .on_press_maybe(get_last_task_input(&self))
                    .style(theme::Button::Text),
                horizontal_space().width(Length::Fill),
                text(format!("Recorded today: {}", get_todays_total_time(&self)))
            ],
            vertical_space().height(Length::Fill),
            text(&self.timer_text)
                .size(80)
                .style(if self.pomodoro.on_break {
                    theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0))
                } else {
                    theme::Text::Default
                }),
            column![
                row![
                    text_input("Task name @Project #tags $rate", &self.task_input)
                        .on_input(Message::TaskInputChanged)
                        .size(20),
                    button(row![
                        horizontal_space().width(Length::Fixed(5.0)),
                        if self.timer_is_running {
                            bootstrap::icon_to_text(bootstrap::Bootstrap::StopFill).size(20)
                        } else {
                            bootstrap::icon_to_text(bootstrap::Bootstrap::PlayFill).size(20)
                        },
                        horizontal_space().width(Length::Fixed(5.0)),
                    ])
                    .on_press_maybe(if self.task_input.trim().is_empty() {
                        None
                    } else {
                        Some(Message::StartStopPressed)
                    })
                ]
                .spacing(10),
                if self.timer_is_running {
                    row![TimePicker::new(
                        self.show_timer_start_picker,
                        self.displayed_task_start_time,
                        Button::new(text(format!(
                            "Started at {}",
                            self.displayed_task_start_time.to_string()
                        )))
                        .on_press(Message::ChooseCurrentTaskStartTime),
                        Message::CancelCurrentTaskStartTime,
                        Message::SubmitCurrentTaskStartTime,
                    )
                    .use_24h(),]
                    .align_items(Alignment::Center)
                    .spacing(10)
                } else {
                    row![button("").style(theme::Button::Text)] // Button to match height
                },
            ]
            .align_items(Alignment::Center)
            .spacing(15),
            vertical_space().height(Length::Fill),
        ]
        .align_items(Alignment::Center)
        .padding(20);

        // MARK: HISTORY
        let mut all_history_rows: Column<'_, Message, Theme, Renderer> =
            Column::new().spacing(8).padding(20);
        if self.inspector_view.is_none() {
            all_history_rows = all_history_rows.push(row![
                horizontal_space(),
                button(bootstrap::icon_to_text(bootstrap::Bootstrap::PlusLg))
                    .on_press(Message::AddNewTaskPressed)
                    .style(theme::Button::Text),
            ]);
        }
        for (date, task_groups) in self.task_history.iter().rev() {
            // let total_time = task_groups
            //     .iter()
            //     .map(|group| group.total_time)
            //     .sum::<i64>();
            let (total_time, total_earnings) = task_groups.iter().fold(
                (0i64, 0f32),
                |(accumulated_time, accumulated_earnings), group| {
                    let group_time = group.total_time;
                    let group_earnings = (group_time as f32 / 3600.0) * group.rate;

                    (
                        accumulated_time + group_time,
                        accumulated_earnings + group_earnings,
                    )
                },
            );
            all_history_rows =
                all_history_rows.push(history_title_row(date, total_time, total_earnings));
            for task_group in task_groups {
                all_history_rows =
                    all_history_rows.push(history_group_row(task_group, self.timer_is_running))
            }
        }
        let history_view = column![Scrollable::new(all_history_rows)
            .width(Length::FillPortion(3)) // TODO: Adjust?
            .height(Length::Fill)];

        // MARK: REPORT
        let report_view = column![Scrollable::new(column![])];

        // MARK: SETTINGS
        let settings_view = column![Tabs::new(Message::SettingsTabSelected)
            .tab_icon_position(iced_aw::tabs::Position::Top)
            .push(
                TabId::General,
                TabLabel::IconText(
                    bootstrap::icon_to_char(Bootstrap::GearFill),
                    "General".to_string()
                ),
                Scrollable::new(
                    column![row![
                        text("Default view"),
                        pick_list(
                            &FurView::ALL[..],
                            Some(self.fur_settings.default_view),
                            Message::SettingsDefaultViewSelected,
                        ),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),]
                    .padding(10)
                ),
            )
            .push(
                TabId::Advanced,
                TabLabel::IconText(
                    bootstrap::icon_to_char(Bootstrap::GearWideConnected),
                    "Advanced".to_string()
                ),
                Scrollable::new(
                    column![
                        row![
                            text("Idle detection"),
                            toggler(
                                String::new(),
                                self.fur_settings.notify_on_idle,
                                Message::SettingsIdleToggled
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Minutes until idle"),
                            number_input(
                                self.fur_settings.chosen_idle_time,
                                999, // TODO: This will accept a range in a future version of iced_aw (make 1..999)
                                Message::SettingsIdleTimeChanged
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                    ]
                    .padding(10)
                ),
            )
            .push(
                TabId::Pomodoro,
                TabLabel::IconText(
                    bootstrap::icon_to_char(Bootstrap::StopwatchFill),
                    "Pomodoro".to_string()
                ),
                Scrollable::new(
                    column![
                        row![
                            text("Countdown timer"),
                            toggler(
                                String::new(),
                                self.fur_settings.pomodoro,
                                Message::SettingsPomodoroToggled
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Timer length"),
                            number_input(
                                self.fur_settings.pomodoro_length,
                                999, // TODO: This will accept a range in a future version of iced_aw (make 1..999)
                                Message::SettingsPomodoroLengthChanged
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Break length"),
                            number_input(
                                self.fur_settings.pomodoro_break_length,
                                999, // TODO: This will accept a range in a future version of iced_aw (make 1..999)
                                Message::SettingsPomodoroBreakLengthChanged
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Snooze length"),
                            number_input(
                                self.fur_settings.pomodoro_snooze_length,
                                999, // TODO: This will accept a range in a future version of iced_aw (make 1..999)
                                Message::SettingsPomodoroSnoozeLengthChanged
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Extended breaks"),
                            toggler(
                                String::new(),
                                self.fur_settings.pomodoro_extended_breaks,
                                Message::SettingsPomodoroExtendedBreaksToggled
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Extended break interval"),
                            number_input(
                                self.fur_settings.pomodoro_extended_break_interval,
                                999, // TODO: This will accept a range in a future version of iced_aw (make 1..999)
                                Message::SettingsPomodoroExtendedBreakIntervalChanged
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                        row![
                            text("Extended break length"),
                            number_input(
                                self.fur_settings.pomodoro_extended_break_length,
                                999, // TODO: This will accept a range in a future version of iced_aw (make 1..999)
                                Message::SettingsPomodoroExtendedBreakLengthChanged
                            )
                            .width(Length::Shrink)
                        ]
                        .spacing(10)
                        .align_items(Alignment::Center),
                    ]
                    .spacing(10)
                    .padding(10),
                ),
            )
            .push(
                TabId::Report,
                TabLabel::IconText(
                    bootstrap::icon_to_char(Bootstrap::GraphUp),
                    "Report".to_string()
                ),
                Scrollable::new(column![].padding(10),),
            )
            .set_active_tab(&self.settings_active_tab)
            .tab_bar_position(TabBarPosition::Top)];

        // MARK: INSPECTOR
        let inspector: Column<'_, Message, Theme, Renderer> = match &self.inspector_view {
            // MARK: Add Task To Group
            Some(FurInspectorView::AddNewTask) => match &self.task_to_add {
                Some(task_to_add) => column![
                    text_input("Task name", &task_to_add.name)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Name)),
                    text_input("Project", &task_to_add.project)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Project)),
                    text_input("#tags", &task_to_add.tags)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags)),
                    text_input("0.00", &task_to_add.new_rate)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Rate)),
                    row![
                        text("Start:"),
                        date_picker(
                            task_to_add.show_start_date_picker,
                            task_to_add.displayed_start_date,
                            button(text(task_to_add.displayed_start_date.to_string())).on_press(
                                Message::ChooseTaskEditDateTime(EditTaskProperty::StartDate)
                            ),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StartDate),
                        ),
                        time_picker(
                            task_to_add.show_start_time_picker,
                            task_to_add.displayed_start_time,
                            Button::new(text(task_to_add.displayed_start_time.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartTime
                                )),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        text("Stop:"),
                        date_picker(
                            task_to_add.show_stop_date_picker,
                            task_to_add.displayed_stop_date,
                            button(text(task_to_add.displayed_stop_date.to_string())).on_press(
                                Message::ChooseTaskEditDateTime(EditTaskProperty::StopDate)
                            ),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StopDate),
                        ),
                        time_picker(
                            task_to_add.show_stop_time_picker,
                            task_to_add.displayed_stop_time,
                            Button::new(text(task_to_add.displayed_stop_time.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopTime
                                )),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        button(text("Cancel").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Secondary)
                            .on_press(Message::CancelTaskEdit)
                            .width(Length::Fill),
                        button(text("Save").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Primary)
                            .on_press_maybe(if task_to_add.name.trim().is_empty() {
                                None
                            } else {
                                Some(Message::SaveTaskEdit)
                            })
                            .width(Length::Fill),
                    ]
                    .padding([20, 0, 0, 0])
                    .spacing(10),
                ]
                .spacing(12)
                .padding(20)
                .width(250)
                .align_items(Alignment::Start),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_items(Alignment::Start),
            },
            Some(FurInspectorView::AddShortcut) => match &self.shortcut_to_add {
                Some(shortcut_to_add) => column![
                    text("New Shortcut").size(24),
                    text_input("Task name", &shortcut_to_add.name)
                        .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Name)),
                    text_input("Project", &shortcut_to_add.project).on_input(|s| {
                        Message::EditShortcutTextChanged(s, EditTaskProperty::Project)
                    }),
                    text_input("#tags", &shortcut_to_add.tags)
                        .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Tags)),
                    row![
                        text("$"),
                        text_input("0.00", &shortcut_to_add.new_rate).on_input(|s| {
                            Message::EditShortcutTextChanged(s, EditTaskProperty::Rate)
                        }),
                        text("/hr"),
                    ]
                    .spacing(3)
                    .align_items(Alignment::Center),
                    color_picker(
                        shortcut_to_add.show_color_picker,
                        shortcut_to_add.color,
                        button(
                            text("Color")
                                .style(if is_dark_color(shortcut_to_add.color.to_srgb()) {
                                    Color::WHITE
                                } else {
                                    Color::BLACK
                                })
                                .width(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .on_press(Message::ChooseShortcutColor)
                        .width(Length::Fill)
                        .style(style::custom_button_style(shortcut_to_add.color.to_srgb(),)),
                        Message::CancelShortcutColor,
                        Message::SubmitShortcutColor,
                    ),
                    row![
                        button(text("Cancel").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Secondary)
                            .on_press(Message::CancelShortcut)
                            .width(Length::Fill),
                        button(text("Save").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Primary)
                            .on_press_maybe(if shortcut_to_add.name.trim().is_empty() {
                                None
                            } else {
                                Some(Message::SaveShortcut)
                            })
                            .width(Length::Fill),
                    ]
                    .padding([20, 0, 0, 0])
                    .spacing(10),
                ]
                .spacing(12)
                .padding(20)
                .width(250)
                .align_items(Alignment::Start),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_items(Alignment::Start),
            },
            Some(FurInspectorView::AddTaskToGroup) => match &self.task_to_add {
                Some(task_to_add) => column![
                    text_input(&task_to_add.name, ""),
                    text_input(&task_to_add.project, ""),
                    text_input(&task_to_add.tags, ""),
                    text_input(&format!("{:.2}", task_to_add.rate), ""),
                    row![
                        text("Start:"),
                        button(
                            text(&task_to_add.displayed_start_date.to_string())
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .on_press_maybe(None),
                        time_picker(
                            task_to_add.show_start_time_picker,
                            task_to_add.displayed_start_time,
                            Button::new(text(task_to_add.displayed_start_time.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartTime
                                )),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        text("Stop:"),
                        button(
                            text(&task_to_add.displayed_stop_date.to_string())
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .on_press_maybe(None),
                        time_picker(
                            task_to_add.show_stop_time_picker,
                            task_to_add.displayed_stop_time,
                            Button::new(text(task_to_add.displayed_stop_time.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopTime
                                )),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        button(text("Cancel").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Secondary)
                            .on_press(Message::CancelTaskEdit)
                            .width(Length::Fill),
                        button(text("Save").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Primary)
                            .on_press(Message::SaveTaskEdit)
                            .width(Length::Fill),
                    ]
                    .padding([20, 0, 0, 0])
                    .spacing(10),
                ]
                .spacing(12)
                .padding(20)
                .width(250)
                .align_items(Alignment::Start),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_items(Alignment::Start),
            },
            // MARK: Edit Single Task
            Some(FurInspectorView::EditTask) => match &self.task_to_edit {
                Some(task_to_edit) => column![
                    row![
                        horizontal_space(),
                        button(bootstrap::icon_to_text(bootstrap::Bootstrap::TrashFill))
                            .on_press(Message::ShowAlert(FurAlert::DeleteTaskConfirmation)) // TODO: if ! delete confirmation run delete only
                            .style(theme::Button::Text),
                    ],
                    text_input(&task_to_edit.name, &task_to_edit.new_name)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Name)),
                    text_input(&task_to_edit.project, &task_to_edit.new_project)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Project)),
                    text_input(&task_to_edit.tags, &task_to_edit.new_tags)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags)),
                    row![
                        text("$"),
                        text_input(
                            &format!("{:.2}", &task_to_edit.rate),
                            &task_to_edit.new_rate
                        )
                        .on_input(|s| { Message::EditTaskTextChanged(s, EditTaskProperty::Rate) }),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        text("Start:"),
                        date_picker(
                            task_to_edit.show_displayed_start_date_picker,
                            task_to_edit.displayed_start_date,
                            button(text(task_to_edit.displayed_start_date.to_string())).on_press(
                                Message::ChooseTaskEditDateTime(EditTaskProperty::StartDate)
                            ),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StartDate),
                        ),
                        time_picker(
                            task_to_edit.show_displayed_start_time_picker,
                            task_to_edit.displayed_start_time,
                            Button::new(text(task_to_edit.displayed_start_time.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartTime
                                )),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        text("Stop:"),
                        date_picker(
                            task_to_edit.show_displayed_stop_date_picker,
                            task_to_edit.displayed_stop_date,
                            button(text(task_to_edit.displayed_stop_date.to_string())).on_press(
                                Message::ChooseTaskEditDateTime(EditTaskProperty::StopDate)
                            ),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StopDate),
                        ),
                        time_picker(
                            task_to_edit.show_displayed_stop_time_picker,
                            task_to_edit.displayed_stop_time,
                            Button::new(text(task_to_edit.displayed_stop_time.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopTime
                                )),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(5),
                    row![
                        button(text("Cancel").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Secondary)
                            .on_press(Message::CancelTaskEdit)
                            .width(Length::Fill),
                        button(text("Save").horizontal_alignment(alignment::Horizontal::Center))
                            .style(theme::Button::Primary)
                            .on_press_maybe(
                                if task_to_edit.is_changed()
                                    && !task_to_edit.new_name.trim().is_empty()
                                {
                                    Some(Message::SaveTaskEdit)
                                } else {
                                    None
                                }
                            )
                            .width(Length::Fill),
                    ]
                    .padding([20, 0, 0, 0])
                    .spacing(10),
                    text(&task_to_edit.invalid_input_error_message)
                        .style(theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0))),
                ]
                .spacing(12)
                .padding(20)
                .width(250)
                .align_items(Alignment::Start),
                None => column![],
            },
            // MARK:: Edit Group
            Some(FurInspectorView::EditGroup) => match &self.group_to_edit {
                Some(group_to_edit) => {
                    let mut group_info_column: Column<'_, Message, Theme, Renderer> =
                        column![text(&group_to_edit.name).font(font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }),]
                        .width(Length::Fill)
                        .align_items(Alignment::Center)
                        .spacing(5)
                        .padding(20);
                    if !group_to_edit.project.is_empty() {
                        group_info_column = group_info_column.push(text(&group_to_edit.project));
                    }
                    if !group_to_edit.tags.is_empty() {
                        group_info_column =
                            group_info_column.push(text(format!("#{}", group_to_edit.tags)));
                    }
                    if group_to_edit.rate != 0.0 {
                        group_info_column =
                            group_info_column.push(text(format!("${}", &group_to_edit.rate)));
                    }
                    let tasks_column: Scrollable<'_, Message, Theme, Renderer> =
                        Scrollable::new(group_to_edit.tasks.iter().fold(
                            Column::new().spacing(5),
                            |column, task| {
                                column
                                    .push(
                                        button(
                                            Container::new(column![
                                                text(format!(
                                                    "{} to {}",
                                                    task.start_time.format("%H:%M").to_string(),
                                                    task.stop_time.format("%H:%M").to_string()
                                                ))
                                                .font(font::Font {
                                                    weight: iced::font::Weight::Bold,
                                                    ..Default::default()
                                                }),
                                                text(format!(
                                                    "Total: {}",
                                                    seconds_to_formatted_duration(
                                                        task.total_time_in_seconds()
                                                    )
                                                ))
                                            ])
                                            .width(Length::Fill)
                                            .padding([5, 8])
                                            .style(style::group_edit_task_row),
                                        )
                                        .on_press(Message::EditTask(task.clone()))
                                        .style(theme::Button::Text),
                                    )
                                    .padding([0, 10, 10, 10])
                            },
                        ));
                    column![
                        row![
                            button(bootstrap::icon_to_text(bootstrap::Bootstrap::XLg))
                                .on_press(Message::CancelGroupEdit)
                                .style(theme::Button::Text),
                            horizontal_space(),
                            button(if group_to_edit.is_in_edit_mode {
                                bootstrap::icon_to_text(bootstrap::Bootstrap::Pencil)
                            } else {
                                bootstrap::icon_to_text(bootstrap::Bootstrap::PencilFill)
                            })
                            .on_press_maybe(if group_to_edit.is_in_edit_mode {
                                None
                            } else {
                                Some(Message::ToggleGroupEditor)
                            })
                            .style(theme::Button::Text),
                            button(bootstrap::icon_to_text(bootstrap::Bootstrap::PlusLg))
                                .on_press_maybe(if group_to_edit.is_in_edit_mode {
                                    None
                                } else {
                                    Some(Message::AddTaskToGroup(group_to_edit.clone()))
                                })
                                .style(theme::Button::Text),
                            button(bootstrap::icon_to_text(bootstrap::Bootstrap::TrashFill))
                                .on_press(Message::ShowAlert(FurAlert::DeleteGroupConfirmation)) // TODO: if ! delete confirmation run delete only
                                .style(theme::Button::Text),
                        ]
                        .spacing(5),
                        match group_to_edit.is_in_edit_mode {
                            true => column![
                                text_input(&group_to_edit.name, &group_to_edit.new_name).on_input(
                                    |s| Message::EditTaskTextChanged(s, EditTaskProperty::Name)
                                ),
                                text_input(&group_to_edit.project, &group_to_edit.new_project)
                                    .on_input(|s| Message::EditTaskTextChanged(
                                        s,
                                        EditTaskProperty::Project
                                    )),
                                text_input(&group_to_edit.tags, &group_to_edit.new_tags).on_input(
                                    |s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags)
                                ),
                                row![
                                    text("$"),
                                    text_input(
                                        &format!("{:.2}", &group_to_edit.rate),
                                        &group_to_edit.new_rate
                                    )
                                    .on_input(|s| {
                                        Message::EditTaskTextChanged(s, EditTaskProperty::Rate)
                                    }),
                                ]
                                .align_items(Alignment::Center)
                                .spacing(5),
                                row![
                                    button(
                                        text("Cancel")
                                            .horizontal_alignment(alignment::Horizontal::Center)
                                    )
                                    .style(theme::Button::Secondary)
                                    .on_press(Message::ToggleGroupEditor)
                                    .width(Length::Fill),
                                    button(
                                        text("Save")
                                            .horizontal_alignment(alignment::Horizontal::Center)
                                    )
                                    .style(theme::Button::Primary)
                                    .on_press_maybe(
                                        if group_to_edit.is_changed()
                                            && !group_to_edit.new_name.trim().is_empty()
                                        {
                                            Some(Message::SaveGroupEdit)
                                        } else {
                                            None
                                        }
                                    )
                                    .width(Length::Fill),
                                ]
                                .padding([20, 0, 0, 0])
                                .spacing(10),
                            ]
                            .padding(20)
                            .spacing(5),
                            false => group_info_column,
                        },
                        tasks_column,
                    ]
                    .spacing(5)
                    .align_items(Alignment::Start)
                }
                None => column![text("Nothing selected.")]
                    .spacing(12)
                    .padding(20)
                    .align_items(Alignment::Start),
            },
            _ => column![],
        };

        let content = row![
            sidebar,
            // Main view
            match self.current_view {
                FurView::Shortcuts => shortcuts_view,
                FurView::Timer => timer_view,
                FurView::History => history_view,
                FurView::Report => report_view,
                FurView::Settings => settings_view,
            },
            inspector.width(if self.inspector_view.is_some() {
                260
            } else {
                0
            }),
        ];

        let overlay: Option<Card<'_, Message, Theme, Renderer>> = if self.displayed_alert.is_some()
        {
            let alert_text: String;
            let alert_description: &str;
            let mut close_button: Option<Button<'_, Message, Theme, Renderer>> = None;
            let mut confirmation_button: Option<Button<'_, Message, Theme, Renderer>> = None;
            let mut snooze_button: Option<Button<'_, Message, Theme, Renderer>> = None;

            match self.displayed_alert.as_ref().unwrap() {
                FurAlert::DeleteGroupConfirmation => {
                    alert_text = "Delete all?".to_string();
                    alert_description =
                        "Are you sure you want to permanently delete all tasks in this group?";
                    close_button = Some(
                        button(
                            text("Cancel")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(theme::Button::Secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text("Delete All")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTasks)
                        .style(theme::Button::Destructive),
                    );
                }
                FurAlert::DeleteTaskConfirmation => {
                    alert_text = "Delete task?".to_string();
                    alert_description = "Are you sure you want to permanently delete this task?";
                    close_button = Some(
                        button(
                            text("Cancel")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(theme::Button::Secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text("Delete")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTasks)
                        .style(theme::Button::Destructive),
                    );
                }
                FurAlert::Idle => {
                    alert_text = format!("You have been idle for {}", self.idle.duration());
                    alert_description =
                        "Would you like to discard that time, or continue the clock?";
                    close_button = Some(
                        button(
                            text("Continue")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::IdleReset)
                        .style(theme::Button::Secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text("Discard")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::IdleDiscard)
                        .style(theme::Button::Destructive),
                    );
                }
                FurAlert::PomodoroBreakOver => {
                    alert_text = "Break's over!".to_string();
                    alert_description = "Time to get back to work.";
                    close_button = Some(
                        button(
                            text("Stop")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStopAfterBreak)
                        .style(theme::Button::Secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text("Continue")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroContinueAfterBreak)
                        .style(theme::Button::Primary),
                    );
                }
                FurAlert::PomodoroOver => {
                    alert_text = "Time's up!".to_string();
                    alert_description = "Are you ready to take a break?";
                    snooze_button = Some(
                        button(
                            // TODO: Try to handle plural with Fluent
                            text(format!(
                                "{} more {}",
                                self.fur_settings.pomodoro_snooze_length,
                                if self.fur_settings.pomodoro_snooze_length > 1 {
                                    "minutes"
                                } else {
                                    "minute"
                                }
                            ))
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .width(Length::Shrink),
                        )
                        .on_press(Message::PomodoroSnooze)
                        .style(theme::Button::Secondary),
                    );
                    close_button = Some(
                        button(
                            text("Stop")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStop)
                        .style(theme::Button::Secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(
                                if self.fur_settings.pomodoro_extended_breaks
                                    && self.pomodoro.sessions
                                        % self.fur_settings.pomodoro_extended_break_interval
                                        == 0
                                {
                                    "Long break"
                                } else {
                                    "Break"
                                },
                            )
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStartBreak)
                        .style(theme::Button::Primary),
                    );
                }
            }

            let mut buttons: Row<'_, Message, Theme, Renderer> =
                row![].spacing(10).padding(5).width(Length::Fill);
            if let Some(more) = snooze_button {
                buttons = buttons.push(more);
            }
            if let Some(close) = close_button {
                buttons = buttons.push(close);
            }
            if let Some(confirmation) = confirmation_button {
                buttons = buttons.push(confirmation);
            }

            Some(
                Card::new(text(alert_text), text(alert_description))
                    .foot(buttons)
                    .max_width(if self.displayed_alert == Some(FurAlert::PomodoroOver) {
                        400.0
                    } else {
                        300.0
                    }),
            )
        } else {
            None
        };

        modal(content, overlay).into()
    }
}

fn nav_button<'a>(text: &'a str, destination: FurView) -> Button<'a, Message> {
    button(text)
        .on_press(Message::NavigateTo(destination))
        .style(theme::Button::Text)
}

fn history_group_row<'a>(task_group: &FurTaskGroup, timer_is_running: bool) -> Button<'a, Message> {
    let mut task_details_column: Column<'_, Message, Theme, Renderer> =
        column![text(&task_group.name).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),]
        .width(Length::FillPortion(6));
    if !task_group.project.is_empty() {
        task_details_column = task_details_column.push(text(format!("@{}", task_group.project)));
    }
    if !task_group.tags.is_empty() {
        task_details_column = task_details_column.push(text(format!("#{}", task_group.tags)));
    }

    let mut task_row: Row<'_, Message, Theme, Renderer> =
        row![].align_items(Alignment::Center).spacing(5);
    if task_group.tasks.len() > 1 {
        task_row = task_row.push(
            Container::new(text(task_group.tasks.len()))
                .align_x(alignment::Horizontal::Center)
                .width(30)
                .style(style::group_count_circle),
        );
    }

    let total_time_str = seconds_to_formatted_duration(task_group.total_time);
    let mut totals_column: Column<'_, Message, Theme, Renderer> = column![text(total_time_str)
        .font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })]
    .align_items(Alignment::End);
    if task_group.rate > 0.0 {
        let total_earnings = task_group.rate * (task_group.total_time as f32 / 3600.0);
        totals_column = totals_column.push(text(&format!("${:.2}", total_earnings)));
    }

    task_row = task_row.push(task_details_column);
    task_row = task_row.push(horizontal_space().width(Length::Fill));
    task_row = task_row.push(totals_column);
    task_row = task_row.push(
        button(bootstrap::icon_to_text(bootstrap::Bootstrap::ArrowRepeat))
            .on_press_maybe(if timer_is_running {
                None
            } else {
                Some(Message::RepeatLastTaskPressed(task_group.to_string()))
            })
            .style(theme::Button::Text),
    );

    button(
        Container::new(task_row)
            .padding([10, 15, 10, 15])
            .width(Length::Fill)
            .style(style::task_row),
    )
    .on_press(Message::EditGroup(task_group.clone()))
    .style(theme::Button::Text)
}

fn history_title_row<'a>(
    date: &NaiveDate,
    total_time: i64,
    total_earnings: f32,
) -> Row<'a, Message> {
    let total_time_str = seconds_to_formatted_duration(total_time);
    let mut total_time_column = column![text(total_time_str).font(font::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    })]
    .align_items(Alignment::End);

    if total_earnings > 0.0 {
        total_time_column = total_time_column.push(text(format!("${:.2}", total_earnings)));
    }

    row![
        text(format_history_date(date)).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        horizontal_space().width(Length::Fill),
        total_time_column,
    ]
    .align_items(Alignment::Center)
}

fn format_history_date(date: &NaiveDate) -> String {
    let today = Local::now().date_naive();
    let yesterday = today - Duration::days(1);
    let current_year = today.year();

    if date == &today {
        "Today".to_string()
    } else if date == &yesterday {
        "Yesterday".to_string()
    } else if date.year() == current_year {
        date.format("%b %d").to_string()
    } else {
        date.format("%b %d, %Y").to_string()
    }
}

fn get_task_history() -> BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> {
    let mut grouped_tasks_by_date: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> = BTreeMap::new();

    //TODO : Change limit based on user limit or max limit. Also should limit by days not items.
    if let Ok(all_tasks) = db_retrieve_all_tasks(SortBy::StopTime, SortOrder::Descending) {
        let tasks_by_date = group_tasks_by_date(all_tasks);

        for (date, tasks) in tasks_by_date {
            let mut all_groups: Vec<FurTaskGroup> = vec![];
            for task in tasks {
                if let Some(matching_group) = all_groups.iter_mut().find(|x| x.is_equal_to(&task)) {
                    matching_group.add(task);
                } else {
                    all_groups.push(FurTaskGroup::new_from(task));
                }
            }
            grouped_tasks_by_date.insert(date, all_groups);
        }
    }
    grouped_tasks_by_date
}

fn group_tasks_by_date(tasks: Vec<FurTask>) -> BTreeMap<chrono::NaiveDate, Vec<FurTask>> {
    let mut grouped_tasks: BTreeMap<chrono::NaiveDate, Vec<FurTask>> = BTreeMap::new();

    for task in tasks {
        let date = task.start_time.date_naive(); // Extract the date part
        grouped_tasks
            .entry(date)
            .or_insert_with(Vec::new)
            .push(task);
    }

    grouped_tasks
}

fn shortcut_button<'a>(timer_is_running: bool, shortcut: &FurShortcut) -> Button<'a, Message> {
    let shortcut_color = match Srgb::from_hex(&shortcut.color_hex) {
        Ok(color) => color,
        Err(_) => Srgb::new(0.694, 0.475, 0.945),
    };
    let text_color = if is_dark_color(shortcut_color) {
        Color::WHITE
    } else {
        Color::BLACK
    };

    button(
        column![
            text(&shortcut.name).style(text_color),
            text(&shortcut.project).style(text_color),
            text(&shortcut.tags).style(text_color),
            text(format!("${:.2}", shortcut.rate)).style(text_color),
        ]
        .width(200)
        .height(170)
        .padding(10),
    )
    .on_press_maybe(if timer_is_running {
        None
    } else {
        Some(Message::ShortcutPressed(shortcut.to_string()))
    })
    .style(style::custom_button_style(shortcut_color))
}

fn is_dark_color(color: Srgb) -> bool {
    color.relative_luminance().luma < 0.65
}

fn start_timer(state: &mut Furtherance) {
    state.timer_start_time = Local::now();
    state.timer_is_running = true;
    if state.fur_settings.pomodoro && !state.pomodoro.on_break {
        state.pomodoro.sessions += 1;
    }
}

fn stop_timer(state: &mut Furtherance, stop_time: DateTime<Local>) {
    state.timer_stop_time = stop_time;
    state.timer_is_running = false;

    let (name, project, tags, rate) = split_task_input(&state.task_input);
    db_write_task(FurTask {
        id: 1, // Not used
        name,
        start_time: state.timer_start_time,
        stop_time: state.timer_stop_time,
        tags,
        project,
        rate,
        currency: String::new(),
    })
    .expect("Couldn't write task to database.");

    reset_timer(state);
}

fn reset_timer(state: &mut Furtherance) {
    state.task_input = "".to_string();
    state.task_history = get_task_history();
    state.timer_text = get_timer_text(state, 0);
    state.idle = FurIdle::new();
}

fn get_timer_text(state: &Furtherance, seconds_elapsed: i64) -> String {
    if state.timer_is_running {
        get_running_timer_text(state, seconds_elapsed)
    } else {
        get_stopped_timer_text(state)
    }
}

fn get_stopped_timer_text(state: &Furtherance) -> String {
    if state.fur_settings.pomodoro {
        if state.pomodoro.on_break {
            if state.fur_settings.pomodoro_extended_breaks
                && state.pomodoro.sessions % state.fur_settings.pomodoro_extended_break_interval
                    == 0
            {
                seconds_to_formatted_duration(
                    state.fur_settings.pomodoro_extended_break_length * 60,
                )
            } else {
                seconds_to_formatted_duration(state.fur_settings.pomodoro_break_length * 60)
            }
        } else if state.pomodoro.snoozed {
            seconds_to_formatted_duration(state.fur_settings.pomodoro_snooze_length * 60)
        } else {
            seconds_to_formatted_duration((state.fur_settings.pomodoro_length * 60))
        }
    } else {
        "0:00:00".to_string()
    }
}

fn get_running_timer_text(state: &Furtherance, seconds_elapsed: i64) -> String {
    if state.fur_settings.pomodoro {
        let stop_time = if state.pomodoro.on_break {
            if state.fur_settings.pomodoro_extended_breaks
                && state.pomodoro.sessions % state.fur_settings.pomodoro_extended_break_interval
                    == 0
            {
                state.timer_start_time
                    + Duration::minutes(state.fur_settings.pomodoro_extended_break_length)
            } else {
                state.timer_start_time + Duration::minutes(state.fur_settings.pomodoro_break_length)
            }
        } else {
            if state.pomodoro.snoozed {
                state.pomodoro.snoozed_at
                    + Duration::minutes(state.fur_settings.pomodoro_snooze_length)
            } else {
                state.timer_start_time + Duration::minutes(state.fur_settings.pomodoro_length)
            }
        };

        let seconds_until_end =
            (stop_time - state.timer_start_time).num_seconds() - seconds_elapsed;
        if seconds_until_end > 0 {
            seconds_to_formatted_duration(seconds_until_end)
        } else {
            "0:00:00".to_string()
        }
    } else {
        seconds_to_formatted_duration(seconds_elapsed)
    }
}

fn convert_timer_text_to_vertical_hms(timer_text: &str) -> String {
    let mut split = timer_text.split(':');
    let mut sidebar_timer_text = String::new();

    if let Some(hours) = split.next() {
        if hours != "0" {
            sidebar_timer_text.push_str(&format!("{} h\n", hours));
        }
    }

    if let Some(mins) = split.next() {
        if mins != "00" {
            sidebar_timer_text.push_str(&format!("{} m\n", mins.trim_start_matches('0')));
        }
    }

    if let Some(secs) = split.next() {
        if secs != "00" {
            sidebar_timer_text.push_str(&format!("{} s", secs.trim_start_matches('0')));
        } else {
            sidebar_timer_text.push_str("0 s");
        }
    }

    sidebar_timer_text
}

fn convert_iced_time_to_chrono_local(iced_time: time_picker::Time) -> LocalResult<DateTime<Local>> {
    let (hour, minute, _) = match iced_time {
        time_picker::Time::Hm {
            hour,
            minute,
            period,
        } => (hour, minute, period),
        _ => (1, 1, time_picker::Period::H24),
    };

    if let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) {
        Local::now().with_time(time)
    } else {
        LocalResult::None
    }
}

async fn get_timer_duration() {
    time::sleep(time::Duration::from_secs(1)).await;
}

pub fn split_task_input(input: &str) -> (String, String, String, f32) {
    let re_name = Regex::new(r"^[^@#$]+").unwrap();
    let re_project = Regex::new(r"@([^#\$]+)").unwrap();
    let re_tags = Regex::new(r"#([^@#$]+)").unwrap();
    let re_rate = Regex::new(r"\$([^@#$]+)").unwrap();

    let name = re_name
        .find(input)
        .map_or("", |m| m.as_str().trim())
        .to_string();

    let project = re_project
        .captures(input)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .unwrap_or(String::new());

    let separated_tags: Vec<String> = re_tags
        .captures_iter(input)
        .map(|cap| cap.get(1).unwrap().as_str().trim().to_string())
        .filter(|s| !s.trim().is_empty())
        .collect();
    let tags = if separated_tags.is_empty() {
        String::new()
    } else {
        separated_tags.join(" #")
    };

    let rate_string = re_rate
        .captures(input)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .unwrap_or("0.0".to_string());
    let rate: f32 = rate_string.parse().unwrap_or(0.0);

    (name, project, tags, rate)
}

fn get_last_task_input(state: &Furtherance) -> Option<Message> {
    if state.timer_is_running {
        None
    } else {
        let today = Local::now().date_naive();
        if let Some(groups) = state.task_history.get(&today) {
            if let Some(last_task) = groups.first() {
                let task_input_builder = last_task.to_string();
                Some(Message::RepeatLastTaskPressed(task_input_builder))
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn get_todays_total_time(state: &Furtherance) -> String {
    let today = Local::now().date_naive();
    let total_seconds: i64 = if let Some(groups) = state.task_history.get(&today) {
        groups.iter().map(|group| group.total_time).sum()
    } else {
        0
    };
    seconds_to_formatted_duration(total_seconds)
}

fn seconds_to_formatted_duration(total_seconds: i64) -> String {
    seconds_to_hms(total_seconds)
    // TODO: If don't show seconds:
    // seconds_to_hm(total_seconds)
}

fn seconds_to_hms(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    let s = total_seconds % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}

fn seconds_to_hm(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    format!("{}:{:02}", h, m)
}

fn combine_chosen_date_with_time(
    old_date_time: DateTime<Local>,
    new_date: date_picker::Date,
) -> LocalResult<DateTime<Local>> {
    Local.with_ymd_and_hms(
        new_date.year,
        new_date.month,
        new_date.day,
        old_date_time.hour(),
        old_date_time.minute(),
        old_date_time.second(),
    )
}

fn combine_chosen_time_with_date(
    old_date_time: DateTime<Local>,
    new_time: time_picker::Time,
) -> LocalResult<DateTime<Local>> {
    let (hour, minute, _) = match new_time {
        time_picker::Time::Hm {
            hour,
            minute,
            period,
        } => (hour, minute, period),
        _ => (1, 1, time_picker::Period::H24),
    };
    Local.with_ymd_and_hms(
        old_date_time.year(),
        old_date_time.month(),
        old_date_time.day(),
        hour,
        minute,
        0,
    )
}

fn has_max_two_decimals(input: &str) -> bool {
    let parts: Vec<&str> = input.split('.').collect();
    match parts.len() {
        1 => true,
        2 => {
            let decimal_part = parts[1];
            decimal_part.len() <= 2
        }
        _ => false,
    }
}
