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

use crate::models::group_to_edit::GroupToEdit;
use crate::models::task_to_add::TaskToAdd;
use crate::models::task_to_edit::TaskToEdit;
use crate::style;
use crate::{
    database::*,
    models::{fur_settings::FurSettings, fur_task::FurTask, fur_task_group::FurTaskGroup},
};
use chrono::{offset::LocalResult, DateTime, Datelike, Local, NaiveDate, NaiveTime};
use chrono::{Duration, TimeZone, Timelike};
use iced::widget::{tooltip, Row};
use iced::Color;
use iced::{
    alignment, font, keyboard,
    multi_window::Application,
    widget::{
        button, column, horizontal_space, pick_list, row, text, text_input, theme, vertical_space,
        Button, Column, Container, Scrollable,
    },
    window, Alignment, Command, Element, Font, Length, Renderer, Settings, Size, Theme,
};
use iced_aw::{
    core::icons::{bootstrap, BOOTSTRAP_FONT_BYTES},
    date_picker, modal, time_picker, Card, Modal, TimePicker,
};
use regex::Regex;
use tokio::time;

#[derive(Debug, Clone, PartialEq)]
pub enum FurView {
    Shortcuts,
    Timer,
    History,
    Report,
    Settings,
}

#[derive(Debug, Clone)]
pub enum FurAlert {
    DeleteGroupConfirmation,
    DeleteTaskConfirmation,
}

#[derive(Debug)]
pub enum FurInspectorView {
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

pub struct Furtherance {
    current_view: FurView,
    displayed_alert: Option<FurAlert>,
    displayed_task_start_time: time_picker::Time,
    group_to_edit: Option<GroupToEdit>,
    inspector_view: Option<FurInspectorView>,
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
    AddTaskToGroup(GroupToEdit),
    AlertClose,
    EditGroup(FurTaskGroup),
    EditTask(FurTask),
    EditTaskTextChanged(String, EditTaskProperty),
    FontLoaded(Result<(), font::Error>),
    CancelCurrentTaskStartTime,
    CancelGroupEdit,
    CancelTaskEdit,
    CancelTaskEditDateTime(EditTaskProperty),
    ChooseCurrentTaskStartTime,
    ChooseTaskEditDateTime(EditTaskProperty),
    DeleteTasks,
    NavigateTo(FurView),
    RepeatLastTaskPressed(String),
    SaveGroupEdit,
    SaveTaskEdit,
    ShowAlert(FurAlert),
    ToggleGroupEditor,
    StartStopPressed,
    StopwatchTick,
    SubmitCurrentTaskStartTime(time_picker::Time),
    SubmitTaskEditDate(date_picker::Date, EditTaskProperty),
    SubmitTaskEditTime(time_picker::Time, EditTaskProperty),
    TaskInputChanged(String),
}

impl Application for Furtherance {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        // Load or create database
        let _ = db_init();
        // Update old furtherance databases with new properties
        let _ = db_upgrade_old();

        let furtherance = Furtherance {
            current_view: FurView::History,
            displayed_alert: None,
            displayed_task_start_time: time_picker::Time::now_hm(true),
            group_to_edit: None,
            inspector_view: None,
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
            Message::AddTaskToGroup(group_to_edit) => {
                self.task_to_add = Some(TaskToAdd::new_from(&group_to_edit));
                self.inspector_view = Some(FurInspectorView::AddTaskToGroup);
                Command::none()
            }
            Message::AlertClose => {
                self.displayed_alert = None;
                Command::none()
            }
            Message::CancelCurrentTaskStartTime => {
                self.show_timer_start_picker = false;
                Command::none()
            }
            Message::CancelGroupEdit => {
                self.group_to_edit = None;
                self.inspector_view = None;
                Command::none()
            }
            Message::CancelTaskEdit => {
                self.task_to_edit = None;
                self.task_to_add = None;
                if self.group_to_edit.is_some() {
                    self.inspector_view = Some(FurInspectorView::EditGroup);
                } else {
                    self.inspector_view = None;
                }
                Command::none()
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
                Command::none()
            }
            Message::ChooseCurrentTaskStartTime => {
                self.show_timer_start_picker = true;
                Command::none()
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
                Command::none()
            }
            Message::DeleteTasks => {
                if let Some(task_to_edit) = &self.task_to_edit {
                    self.inspector_view = None;
                    let _ = db_delete_by_ids(vec![task_to_edit.id]);
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history();
                } else if let Some(group_to_edit) = &self.group_to_edit {
                    self.inspector_view = None;
                    let _ = db_delete_by_ids(group_to_edit.task_ids());
                    self.group_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history();
                }
                Command::none()
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
                Command::none()
            }
            Message::EditTask(task) => {
                self.task_to_edit = Some(TaskToEdit::new_from(&task));
                self.inspector_view = Some(FurInspectorView::EditTask);
                Command::none()
            }
            Message::EditTaskTextChanged(new_value, property) => {
                match self.inspector_view {
                    Some(FurInspectorView::EditTask) => {
                        if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                            match property {
                                EditTaskProperty::Name => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        task_to_edit.invalid_input_error_message =
                                            "Task name cannot contain #, @, or $.".to_string();
                                    } else {
                                        task_to_edit.new_name = new_value;
                                        task_to_edit.invalid_input_error_message = String::new();
                                    }
                                }
                                EditTaskProperty::Project => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        // TODO: Change to .input_error system
                                        task_to_edit.invalid_input_error_message =
                                            "Project cannot contain #, @, or $.".to_string();
                                    } else {
                                        task_to_edit.new_project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        task_to_edit.invalid_input_error_message =
                                            "Tags cannot contain @ or $.".to_string();
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        task_to_edit.invalid_input_error_message =
                                            "Tags must start with a #.".to_string();
                                    } else {
                                        task_to_edit.new_tags = new_value;
                                        task_to_edit.invalid_input_error_message = String::new();
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        task_to_edit.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        task_to_edit.invalid_input_error_message =
                                            "Do not include a $ in the rate.".to_string();
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(
                                            new_value_parsed.clone().unwrap_or(0.0),
                                        )
                                        && new_value_parsed.unwrap_or(f32::INFINITY).is_finite()
                                    {
                                        task_to_edit.new_rate = new_value;
                                        task_to_edit.invalid_input_error_message = String::new();
                                    } else {
                                        task_to_edit.invalid_input_error_message =
                                            "Rate must be a valid dollar amount.".to_string();
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
                                        && has_max_two_decimals(
                                            new_value_parsed.clone().unwrap_or(0.0),
                                        )
                                        && new_value_parsed.unwrap_or(f32::INFINITY).is_finite()
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
                Command::none()
            }
            Message::FontLoaded(_) => Command::none(),
            Message::NavigateTo(destination) => {
                if self.current_view != destination {
                    self.inspector_view = None;
                    self.current_view = destination;
                }
                Command::none()
            }
            Message::RepeatLastTaskPressed(last_task_input) => {
                self.task_input = last_task_input;
                self.current_view = FurView::Timer;
                Command::perform(async { Message::StartStopPressed }, |msg| msg)
            }
            Message::SaveGroupEdit => {
                if let Some(group_to_edit) = &self.group_to_edit {
                    let _ = db_update_group_of_tasks(group_to_edit);
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history();
                }
                Command::none()
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
                    });
                    self.inspector_view = None;
                    self.task_to_add = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history();
                }
                Command::none()
            }
            Message::ShowAlert(alert_to_show) => {
                self.displayed_alert = Some(alert_to_show);
                Command::none()
            }
            Message::StartStopPressed => {
                if self.timer_is_running {
                    // Stop & reset timer
                    self.timer_stop_time = Local::now();
                    self.timer_is_running = false;

                    let (name, project, tags, rate) = split_task_input(&self.task_input);
                    db_write_task(FurTask {
                        id: 1, // Not used
                        name,
                        start_time: self.timer_start_time,
                        stop_time: self.timer_stop_time,
                        tags,
                        project,
                        rate,
                    })
                    .expect("Couldn't write task to database.");

                    self.task_input = "".to_string();
                    self.task_history = get_task_history();
                    self.timer_text = "0:00:00".to_string();
                    Command::none()
                } else {
                    // Start timer
                    self.timer_start_time = Local::now();
                    self.timer_is_running = true;
                    Command::perform(get_timer_duration(), |_| Message::StopwatchTick)
                }
            }
            Message::StopwatchTick => {
                if self.timer_is_running {
                    let duration = Local::now().signed_duration_since(self.timer_start_time);
                    let hours = duration.num_hours();
                    let minutes = duration.num_minutes() % 60;
                    let seconds = duration.num_seconds() % 60;
                    self.timer_text = format!("{}:{:02}:{:02}", hours, minutes, seconds);

                    Command::perform(get_timer_duration(), |_| Message::StopwatchTick)
                } else {
                    Command::none()
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
                Command::none()
            }
            Message::SubmitTaskEditDate(new_date, property) => {
                if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                    if let LocalResult::Single(new_local_date_time) =
                        combine_chosen_date_with_time(task_to_edit.new_start_time, new_date)
                    {
                        if new_local_date_time <= Local::now() {
                            match property {
                                EditTaskProperty::StartDate => {
                                    task_to_edit.displayed_start_date = new_date;
                                    task_to_edit.new_start_time = new_local_date_time;
                                    task_to_edit.show_displayed_start_date_picker = false;
                                }
                                EditTaskProperty::StopDate => {
                                    task_to_edit.displayed_stop_date = new_date;
                                    task_to_edit.new_stop_time = new_local_date_time;
                                    task_to_edit.show_displayed_stop_date_picker = false;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Command::none()
            }
            Message::SubmitTaskEditTime(new_time, property) => {
                // TODO: Edit to fix issues in greater than stop, etc. like below
                if let Some(task_to_edit) = self.task_to_edit.as_mut() {
                    if let LocalResult::Single(new_local_date_time) =
                        combine_chosen_time_with_date(task_to_edit.new_start_time, new_time)
                    {
                        if new_local_date_time <= Local::now() {
                            match property {
                                EditTaskProperty::StartTime => {
                                    task_to_edit.displayed_start_time = new_time;
                                    task_to_edit.new_start_time = new_local_date_time;
                                    task_to_edit.show_displayed_start_time_picker = false;
                                }
                                EditTaskProperty::StopTime => {
                                    task_to_edit.displayed_stop_time = new_time;
                                    task_to_edit.new_stop_time = new_local_date_time;
                                    task_to_edit.show_displayed_stop_time_picker = false;
                                }
                                _ => {}
                            }
                        }
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
                Command::none()
            }
            Message::TaskInputChanged(new_value) => {
                // Handle all possible task input checks here rather than on start/stop press
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
                                && has_max_two_decimals(parsed_num.clone().unwrap_or(0.0))
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
                Command::none()
            }
            Message::ToggleGroupEditor => {
                self.group_to_edit
                    .as_mut()
                    .map(|group| group.is_in_edit_mode = !group.is_in_edit_mode);
                Command::none()
            }
        }
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
                // TODO: if timer is running and nav is not timer, show timer
                nav_button("Settings", FurView::Settings)
            ]
            .spacing(12)
            .padding(20)
            .width(175)
            .align_items(Alignment::Start),
        )
        .style(style::gray_background);

        // MARK: Shortcuts
        let shortcuts_view = column![Scrollable::new(column![])];

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
            text(&self.timer_text).size(80),
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
                    .on_press(Message::StartStopPressed)
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
        for (date, task_groups) in self.task_history.iter().rev() {
            let total_time = task_groups
                .iter()
                .map(|group| group.total_time)
                .sum::<i64>();
            all_history_rows = all_history_rows.push(history_title_row(date, total_time));
            for task_group in task_groups {
                all_history_rows = all_history_rows.push(history_group_row(task_group))
            }
        }
        let history_view = column![Scrollable::new(all_history_rows)
            .width(Length::FillPortion(3)) // TODO: Adjust?
            .height(Length::Fill)];

        // MARK: REPORT
        let report_view = column![Scrollable::new(column![])];

        // MARK: SETTINGS
        let settings_view = column![Scrollable::new(column![])];

        // MARK: INSPECTOR
        let inspector: Container<'_, Message, Theme, Renderer> =
            Container::new(match &self.inspector_view {
                // MARK: Add Task To Group
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
                                |time| Message::SubmitTaskEditTime(
                                    time,
                                    EditTaskProperty::StartTime
                                ),
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
                                |time| Message::SubmitTaskEditTime(
                                    time,
                                    EditTaskProperty::StopTime
                                ),
                            )
                            .use_24h(),
                        ]
                        .align_items(Alignment::Center)
                        .spacing(5),
                        row![
                            button(
                                text("Cancel").horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .style(theme::Button::Secondary)
                            .on_press(Message::CancelTaskEdit)
                            .width(Length::Fill),
                            button(
                                text("Save").horizontal_alignment(alignment::Horizontal::Center)
                            )
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
                        text_input(&task_to_edit.project, &task_to_edit.new_project).on_input(
                            |s| Message::EditTaskTextChanged(s, EditTaskProperty::Project)
                        ),
                        text_input(&task_to_edit.tags, &task_to_edit.new_tags)
                            .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags)),
                        row![
                            text("$"),
                            text_input(
                                &format!("{:.2}", &task_to_edit.rate),
                                &task_to_edit.new_rate
                            )
                            .on_input(|s| {
                                Message::EditTaskTextChanged(s, EditTaskProperty::Rate)
                            }),
                        ]
                        .align_items(Alignment::Center)
                        .spacing(5),
                        row![
                            text("Start:"),
                            date_picker(
                                task_to_edit.show_displayed_start_date_picker,
                                task_to_edit.displayed_start_date,
                                button(text(task_to_edit.displayed_start_date.to_string()))
                                    .on_press(Message::ChooseTaskEditDateTime(
                                        EditTaskProperty::StartDate
                                    )),
                                Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                                |date| Message::SubmitTaskEditDate(
                                    date,
                                    EditTaskProperty::StartDate
                                ),
                            ),
                            time_picker(
                                task_to_edit.show_displayed_start_time_picker,
                                task_to_edit.displayed_start_time,
                                Button::new(text(task_to_edit.displayed_start_time.to_string()))
                                    .on_press(Message::ChooseTaskEditDateTime(
                                        EditTaskProperty::StartTime
                                    )),
                                Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                                |time| Message::SubmitTaskEditTime(
                                    time,
                                    EditTaskProperty::StartTime
                                ),
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
                                button(text(task_to_edit.displayed_stop_date.to_string()))
                                    .on_press(Message::ChooseTaskEditDateTime(
                                        EditTaskProperty::StopDate
                                    )),
                                Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                                |date| Message::SubmitTaskEditDate(
                                    date,
                                    EditTaskProperty::StopDate
                                ),
                            ),
                            time_picker(
                                task_to_edit.show_displayed_stop_time_picker,
                                task_to_edit.displayed_stop_time,
                                Button::new(text(task_to_edit.displayed_stop_time.to_string()))
                                    .on_press(Message::ChooseTaskEditDateTime(
                                        EditTaskProperty::StopTime
                                    )),
                                Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                                |time| Message::SubmitTaskEditTime(
                                    time,
                                    EditTaskProperty::StopTime
                                ),
                            )
                            .use_24h(),
                        ]
                        .align_items(Alignment::Center)
                        .spacing(5),
                        row![
                            button(
                                text("Cancel").horizontal_alignment(alignment::Horizontal::Center)
                            )
                            .style(theme::Button::Secondary)
                            .on_press(Message::CancelTaskEdit)
                            .width(Length::Fill),
                            button(
                                text("Save").horizontal_alignment(alignment::Horizontal::Center)
                            )
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
                            group_info_column =
                                group_info_column.push(text(&group_to_edit.project));
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
                                    text_input(&group_to_edit.name, &group_to_edit.new_name)
                                        .on_input(|s| Message::EditTaskTextChanged(
                                            s,
                                            EditTaskProperty::Name
                                        )),
                                    text_input(&group_to_edit.project, &group_to_edit.new_project)
                                        .on_input(|s| Message::EditTaskTextChanged(
                                            s,
                                            EditTaskProperty::Project
                                        )),
                                    text_input(&group_to_edit.tags, &group_to_edit.new_tags)
                                        .on_input(|s| Message::EditTaskTextChanged(
                                            s,
                                            EditTaskProperty::Tags
                                        )),
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
                                            text("Cancel").horizontal_alignment(
                                                alignment::Horizontal::Center
                                            )
                                        )
                                        .style(theme::Button::Secondary)
                                        .on_press(Message::ToggleGroupEditor)
                                        .width(Length::Fill),
                                        button(
                                            text("Save").horizontal_alignment(
                                                alignment::Horizontal::Center
                                            )
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
                        .width(250)
                        .align_items(Alignment::Start)
                    }
                    None => column![text("Nothing selected.")]
                        .spacing(12)
                        .padding(20)
                        .width(175)
                        .align_items(Alignment::Start),
                },
                _ => column![],
            });

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
            inspector,
        ];

        let overlay: Option<Card<'_, Message, Theme, Renderer>> = if self.displayed_alert.is_some()
        {
            let alert_text: &str;
            let alert_description: &str;
            let close_button_text: &str;
            let mut close_button_style: theme::Button = theme::Button::Primary;
            let mut confirmation_button: Option<Button<'_, Message, Theme, Renderer>> = None;

            match self.displayed_alert.as_ref().unwrap() {
                FurAlert::DeleteGroupConfirmation => {
                    alert_text = "Delete all?";
                    alert_description =
                        "Are you sure you want to permanently delete all tasks in this group?";
                    close_button_text = "Cancel";
                    close_button_style = theme::Button::Secondary;
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
                    alert_text = "Delete task?";
                    alert_description = "Are you sure you want to permanently delete this task?";
                    close_button_text = "Cancel";
                    close_button_style = theme::Button::Secondary;
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
            }

            let mut buttons: Row<'_, Message, Theme, Renderer> = row![button(
                text(close_button_text)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .width(Length::Fill)
            )
            .on_press(Message::AlertClose)
            .style(close_button_style)]
            .spacing(10)
            .padding(5)
            .width(Length::Fill);

            if let Some(confirmation) = confirmation_button {
                buttons = buttons.push(confirmation);
            }

            Some(
                Card::new(text(alert_text), text(alert_description))
                    .foot(buttons)
                    .max_width(300.0)
                    .on_close(Message::AlertClose),
            )
        } else {
            None
        };

        modal(content, overlay)
            .backdrop(Message::AlertClose)
            .on_esc(Message::AlertClose)
            .into()
    }
}

fn nav_button<'a>(text: &'a str, destination: FurView) -> Button<'a, Message> {
    button(text)
        .on_press(Message::NavigateTo(destination))
        .style(theme::Button::Text)
}

fn history_group_row<'a>(task_group: &FurTaskGroup) -> Button<'a, Message> {
    let total_time_str = seconds_to_formatted_duration(task_group.total_time);
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
    task_row = task_row.push(task_details_column);
    task_row = task_row.push(horizontal_space().width(Length::Fill));
    task_row = task_row.push(text(total_time_str));
    task_row = task_row.push(
        button(bootstrap::icon_to_text(bootstrap::Bootstrap::ArrowRepeat))
            .on_press(Message::RepeatLastTaskPressed(task_input_builder(
                task_group,
            )))
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

// fn get_task_group_with_id(state: &Furtherance) -> Option<&FurTaskGroup> {
//     for value in state.task_history.values() {
//         if let Some(group_to_edit) = value.iter().find(|v| v.id == state.group_id_to_edit) {
//             return Some(group_to_edit);
//         }
//     }
//     None
// }

// fn get_mutable_task_group_with_id(state: &mut Furtherance) -> Option<&mut FurTaskGroup> {
//     for value in map.values_mut() {
//         if value.id == target_id {
//             return Some(value);
//         }
//     }
//     None
// }

fn history_title_row<'a>(date: &NaiveDate, total_time: i64) -> Row<'a, Message> {
    let total_time_str = seconds_to_formatted_duration(total_time);
    row![
        text(format_history_date(date)).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        horizontal_space().width(Length::Fill),
        text(total_time_str).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
    ]
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
    if let Ok(all_tasks) = db_retrieve_all(SortBy::StopTime, SortOrder::Descending) {
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

fn hex_to_color(hex: &str) -> Color {
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();

    Color::from_rgb8(r, g, b)
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
    let today = Local::now().date_naive();
    if let Some(groups) = state.task_history.get(&today) {
        if let Some(last_task) = groups.first() {
            let task_input_builder = task_input_builder(last_task);
            Some(Message::RepeatLastTaskPressed(task_input_builder))
        } else {
            None
        }
    } else {
        None
    }
}

// TODO: Use task.to_string instead
fn task_input_builder(task_group: &FurTaskGroup) -> String {
    let mut task_input_builder = task_group.name.to_string();

    if !task_group.project.is_empty() {
        task_input_builder += &format!(" @{}", task_group.project);
    }

    if !task_group.tags.is_empty() {
        task_input_builder += &format!(" #{}", task_group.tags);
    }

    if task_group.rate != 0.0 {
        task_input_builder += &format!(" ${}", task_group.rate);
    }

    task_input_builder
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

fn has_max_two_decimals(num: f32) -> bool {
    let shifted = num * 100.0;
    let rounded = shifted.round();
    (shifted - rounded).abs() < f32::EPSILON
}
