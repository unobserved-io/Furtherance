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
use std::{
    collections::BTreeMap,
    fs::File,
    io::Seek,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{
    constants::{ALLOWED_DB_EXTENSIONS, SETTINGS_SPACING},
    database::*,
    helpers::{
        color_utils::{FromHex, RandomColor, ToHex, ToSrgb},
        flow_row::FlowRow,
        idle::mac_win_idle::get_idle_time,
    },
    localization::Localization,
    models::{
        fur_idle::FurIdle, fur_pomodoro::FurPomodoro, fur_report::FurReport,
        fur_settings::FurSettings, fur_shortcut::FurShortcut, fur_task::FurTask,
        fur_task_group::FurTaskGroup, group_to_edit::GroupToEdit, shortcut_to_add::ShortcutToAdd,
        shortcut_to_edit::ShortcutToEdit, task_to_add::TaskToAdd, task_to_edit::TaskToEdit,
    },
    style,
    view_enums::*,
};
use chrono::{offset::LocalResult, DateTime, Datelike, Local, NaiveDate, NaiveTime};
use chrono::{TimeDelta, TimeZone, Timelike};
use csv::{Reader, ReaderBuilder, StringRecord, Writer};
use fluent::{FluentBundle, FluentResource};
use iced::{
    alignment, font,
    multi_window::Application,
    widget::{
        button, column, horizontal_space, pick_list, row, text, text_input, theme, vertical_space,
        Button, Column, Container, Scrollable,
    },
    window, Alignment, Command, Element, Length, Renderer, Theme,
};
use iced::{widget::vertical_rule, Color};
use iced::{
    widget::{checkbox, horizontal_rule, toggler, Row},
    Subscription,
};
use iced_aw::{
    color_picker,
    core::icons::{bootstrap, Bootstrap, BOOTSTRAP_FONT_BYTES},
    date_picker, modal, number_input, time_picker, Card, ContextMenu, TabBarPosition, TabLabel,
    Tabs, TimePicker,
};
use notify_rust::{Notification, Timeout};
use palette::color_difference::Wcag21RelativeContrast;
use palette::Srgb;
use regex::Regex;
use rfd::FileDialog;
use tokio::time::{self, interval_at};

#[cfg(not(target_os = "macos"))]
use iced::Subscription;

pub struct Furtherance {
    current_view: FurView,
    delete_ids_from_context: Option<Vec<u32>>,
    delete_shortcut_from_context: Option<u32>,
    displayed_alert: Option<FurAlert>,
    displayed_task_start_time: time_picker::Time,
    fur_settings: FurSettings,
    group_to_edit: Option<GroupToEdit>,
    idle: FurIdle,
    inspector_view: Option<FurInspectorView>,
    localization: Arc<Localization>,
    pomodoro: FurPomodoro,
    report: FurReport,
    settings_active_tab: TabId,
    settings_backup_message: Result<String, Box<dyn std::error::Error>>,
    settings_csv_message: Result<String, Box<dyn std::error::Error>>,
    settings_database_error: Result<String, Box<dyn std::error::Error>>,
    shortcuts: Vec<FurShortcut>,
    shortcut_to_add: Option<ShortcutToAdd>,
    shortcut_to_edit: Option<ShortcutToEdit>,
    show_timer_start_picker: bool,
    task_history: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>>,
    task_input: String,
    theme: Theme,
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
    BackupDatabase,
    CancelCurrentTaskStartTime,
    CancelEndDate,
    CancelGroupEdit,
    CancelShortcut,
    CancelShortcutColor,
    CancelStartDate,
    CancelTaskEdit,
    CancelTaskEditDateTime(EditTaskProperty),
    ChangeTheme,
    ChartTaskPropertyKeySelected(FurTaskProperty),
    ChartTaskPropertyValueSelected(String),
    ChooseCurrentTaskStartTime,
    ChooseEndDate,
    ChooseShortcutColor,
    ChooseStartDate,
    ChooseTaskEditDateTime(EditTaskProperty),
    CreateShortcutFromTaskGroup(FurTaskGroup),
    DateRangeSelected(FurDateRange),
    DeleteShortcut,
    DeleteShortcutFromContext(u32),
    DeleteTasks,
    DeleteTasksFromContext(Vec<u32>),
    EditGroup(FurTaskGroup),
    EditShortcutPressed(FurShortcut),
    EditShortcutTextChanged(String, EditTaskProperty),
    EditTask(FurTask),
    EditTaskTextChanged(String, EditTaskProperty),
    EnterPressedInTaskInput,
    ExportCsvPressed,
    FontLoaded(Result<(), font::Error>),
    IdleDiscard,
    IdleReset,
    ImportCsvPressed,
    MidnightReached,
    NavigateTo(FurView),
    PomodoroContinueAfterBreak,
    PomodoroSnooze,
    PomodoroStartBreak,
    PomodoroStop,
    PomodoroStopAfterBreak,
    RepeatLastTaskPressed(String),
    ReportTabSelected(TabId),
    SaveGroupEdit,
    SaveShortcut,
    SaveTaskEdit,
    SettingsChangeDatabaseLocationPressed(ChangeDB),
    SettingsDatabaseLocationInputChanged(String),
    SettingsDaysToShowChanged(i64),
    SettingsDefaultViewSelected(FurView),
    SettingsDeleteConfirmationToggled(bool),
    SettingsDynamicTotalToggled(bool),
    SettingsIdleTimeChanged(i64),
    SettingsIdleToggled(bool),
    SettingsPomodoroBreakLengthChanged(i64),
    SettingsPomodoroExtendedBreaksToggled(bool),
    SettingsPomodoroExtendedBreakIntervalChanged(u16),
    SettingsPomodoroExtendedBreakLengthChanged(i64),
    SettingsPomodoroLengthChanged(i64),
    SettingsPomodoroSnoozeLengthChanged(i64),
    SettingsPomodoroToggled(bool),
    SettingsShowChartAverageEarningsToggled(bool),
    SettingsShowChartAverageTimeToggled(bool),
    SettingsShowChartBreakdownBySelectionToggled(bool),
    SettingsShowChartEarningsToggled(bool),
    SettingsShowChartSelectionEarningsToggled(bool),
    SettingsShowChartSelectionTimeToggled(bool),
    SettingsShowChartTimeRecordedToggled(bool),
    SettingsShowChartTotalEarningsBoxToggled(bool),
    SettingsShowChartTotalTimeBoxToggled(bool),
    SettingsShowProjectToggled(bool),
    SettingsShowTagsToggled(bool),
    SettingsShowEarningsToggled(bool),
    SettingsShowSecondsToggled(bool),
    SettingsShowDailyTimeTotalToggled(bool),
    SettingsTabSelected(TabId),
    ShortcutPressed(String),
    ShowAlert(FurAlert),
    StartStopPressed,
    StopwatchTick,
    SubmitCurrentTaskStartTime(time_picker::Time),
    SubmitEndDate(date_picker::Date),
    SubmitShortcutColor(Color),
    SubmitStartDate(date_picker::Date),
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
        let mut settings = match FurSettings::new() {
            Ok(loaded_settings) => loaded_settings,
            Err(e) => {
                eprintln!("Error loading settings: {}", e);
                FurSettings::default()
            }
        };
        // Load or create database
        if let Err(e) = db_init() {
            if let Err(e) = settings.reset_to_default_db_location() {
                eprintln!("Error loading database. Can't load or save data: {}", e);
            }
            eprintln!(
                "Error loading database. Reverting to default location: {}",
                e
            );
        }
        // Update old furtherance databases with new properties
        if let Err(e) = db_upgrade_old() {
            eprintln!("Error encountered while upgrading legacy database: {}", e);
        }

        let mut furtherance = Furtherance {
            current_view: settings.default_view,
            delete_ids_from_context: None,
            delete_shortcut_from_context: None,
            displayed_alert: None,
            displayed_task_start_time: time_picker::Time::now_hm(true),
            fur_settings: settings,
            group_to_edit: None,
            idle: FurIdle::new(),
            localization: Arc::new(Localization::new()),
            pomodoro: FurPomodoro::new(),
            inspector_view: None,
            report: FurReport::new(),
            settings_active_tab: TabId::General,
            settings_backup_message: Ok(String::new()),
            settings_csv_message: Ok(String::new()),
            settings_database_error: Ok(String::new()),
            shortcuts: match db_retrieve_shortcuts() {
                Ok(shortcuts) => shortcuts,
                Err(e) => {
                    eprintln!("Error reading shortcuts from database: {}", e);
                    vec![]
                }
            },
            shortcut_to_add: None,
            shortcut_to_edit: None,
            show_timer_start_picker: false,
            task_history: BTreeMap::<chrono::NaiveDate, Vec<FurTaskGroup>>::new(),
            task_input: "".to_string(),
            theme: get_system_theme(),
            timer_is_running: false,
            timer_start_time: Local::now(),
            timer_stop_time: Local::now(),
            timer_text: "0:00:00".to_string(),
            task_to_add: None,
            task_to_edit: None,
        };

        furtherance.timer_text = get_timer_text(&furtherance, 0);
        furtherance.task_history = get_task_history(furtherance.fur_settings.days_to_show);

        (
            furtherance,
            font::load(BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
        )
    }

    fn title(&self, _window_id: window::Id) -> String {
        "Furtherance".to_owned()
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Live dark-light theme switching does not currently work on macOS
        #[cfg(not(target_os = "macos"))]
        let theme_watcher =
            iced::time::every(time::Duration::from_secs(1)).map(|_| Message::ChangeTheme);

        // Watch for midnight to update the history
        struct MidnightSub;
        let midnight_subscription =
            iced::subscription::unfold(std::any::TypeId::of::<MidnightSub>(), (), |_| async {
                let now = Local::now();
                let next_midnight = (now + chrono::Duration::days(1))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap();
                let duration_until_midnight = next_midnight - now;
                let tokio_instant = tokio::time::Instant::now()
                    + Duration::from_secs(duration_until_midnight.num_seconds() as u64);

                let mut interval = interval_at(tokio_instant, Duration::from_secs(24 * 60 * 60));
                interval.tick().await; // Wait for the first tick (midnight)

                (Message::MidnightReached, ())
            });

        Subscription::batch([
            midnight_subscription,
            #[cfg(not(target_os = "macos"))]
            theme_watcher,
        ])
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
            Message::AlertClose => {
                self.delete_ids_from_context = None;
                self.delete_shortcut_from_context = None;
                self.displayed_alert = None;
            }
            Message::BackupDatabase => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_error = Ok(String::new());
                let file_name = format!("furtherance-bkup-{}.db", Local::now().format("%Y-%m-%d"));
                let selected_file = FileDialog::new()
                    .set_title(self.localization.get_message("save-backup-title", None))
                    .add_filter(
                        self.localization.get_message("sqlite-database", None),
                        &["db"],
                    )
                    .set_can_create_directories(true)
                    .set_file_name(file_name)
                    .save_file();

                if let Some(file) = selected_file {
                    match db_backup(file) {
                        Ok(_) => {
                            self.settings_backup_message =
                                Ok(self.localization.get_message("backup-successful", None));
                        }
                        Err(_) => {
                            self.settings_backup_message = Err(self.localization.get_message("backup-database-failed", None).into());
                        }
                    }
                }
            }
            Message::CancelCurrentTaskStartTime => self.show_timer_start_picker = false,
            Message::CancelEndDate => self.report.show_end_date_picker = false,
            Message::CancelGroupEdit => {
                self.group_to_edit = None;
                self.inspector_view = None;
            }
            Message::CancelShortcut => {
                self.shortcut_to_add = None;
                self.shortcut_to_edit = None;
                self.inspector_view = None;
            }
            Message::CancelStartDate => self.report.show_start_date_picker = false,
            Message::CancelShortcutColor => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    shortcut_to_add.show_color_picker = false;
                } else if let Some(shortcut_to_edit) = self.shortcut_to_edit.as_mut() {
                    shortcut_to_edit.show_color_picker = false;
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
            Message::ChangeTheme => self.theme = get_system_theme(),
            Message::ChartTaskPropertyKeySelected(new_property) => {
                self.report.set_picked_task_property_key(new_property);
            }
            Message::ChartTaskPropertyValueSelected(new_value) => {
                self.report.set_picked_task_property_value(new_value);
            }
            Message::ChooseCurrentTaskStartTime => self.show_timer_start_picker = true,
            Message::ChooseEndDate => self.report.show_end_date_picker = true,
            Message::ChooseShortcutColor => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    shortcut_to_add.show_color_picker = true
                } else if let Some(shortcut_to_edit) = self.shortcut_to_edit.as_mut() {
                    shortcut_to_edit.show_color_picker = true
                }
            }
            Message::ChooseStartDate => self.report.show_start_date_picker = true,
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
            Message::CreateShortcutFromTaskGroup(task_group) => {
                let new_shortcut = FurShortcut {
                    id: 0,
                    name: task_group.name,
                    tags: if task_group.tags.is_empty() {
                        String::new()
                    } else {
                        format!("#{}", task_group.tags)
                    },
                    project: task_group.project,
                    rate: task_group.rate,
                    currency: String::new(),
                    color_hex: Srgb::random().to_hex(),
                };

                match db_shortcut_exists(&new_shortcut) {
                    Ok(exists) => {
                        if exists {
                            self.displayed_alert = Some(FurAlert::ShortcutExists);
                        } else {
                            if let Err(e) = db_write_shortcut(new_shortcut) {
                                eprintln!("Failed to write shortcut to database: {}", e);
                            }
                            match db_retrieve_shortcuts() {
                                Ok(shortcuts) => self.shortcuts = shortcuts,
                                Err(e) => {
                                    eprintln!("Failed to retrieve shortcuts from database: {}", e)
                                }
                            };
                            self.current_view = FurView::Shortcuts;
                        }
                    }
                    Err(e) => eprintln!("Failed to check if shortcut exists: {}", e),
                }
            }
            Message::DateRangeSelected(new_range) => self.report.set_picked_date_ranged(new_range),
            Message::DeleteShortcut => {
                if let Some(id) = self.delete_shortcut_from_context {
                    if let Err(e) = db_delete_shortcut_by_id(id) {
                        eprintln!("Failed to delete shortcut: {}", e);
                    }
                    self.delete_shortcut_from_context = None;
                    self.displayed_alert = None;
                    match db_retrieve_shortcuts() {
                        Ok(shortcuts) => self.shortcuts = shortcuts,
                        Err(e) => eprintln!("Failed to retrieve shortcuts from database: {}", e),
                    };
                }
            }
            Message::DeleteShortcutFromContext(id) => {
                self.delete_shortcut_from_context = Some(id);
                let delete_confirmation = self.fur_settings.show_delete_confirmation;
                return Command::perform(
                    async move {
                        if delete_confirmation {
                            Message::ShowAlert(FurAlert::DeleteShortcutConfirmation)
                        } else {
                            Message::DeleteShortcut
                        }
                    },
                    |msg| msg,
                );
            }
            Message::DeleteTasks => {
                if let Some(tasks_to_delete) = &self.delete_ids_from_context {
                    if let Err(e) = db_delete_tasks_by_ids(tasks_to_delete.clone()) {
                        eprintln!("Failed to delete tasks: {}", e);
                    }
                    self.delete_ids_from_context = None;
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                } else if let Some(task_to_edit) = &self.task_to_edit {
                    self.inspector_view = None;
                    if let Err(e) = db_delete_tasks_by_ids(vec![task_to_edit.id]) {
                        eprintln!("Failed to delete task: {}", e);
                    }
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                } else if let Some(group_to_edit) = &self.group_to_edit {
                    self.inspector_view = None;
                    if let Err(e) = db_delete_tasks_by_ids(group_to_edit.task_ids()) {
                        eprintln!("Failed to delete tasks: {}", e);
                    }
                    self.group_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                }
            }
            Message::DeleteTasksFromContext(task_group_ids) => {
                let number_of_tasks = task_group_ids.len();
                self.delete_ids_from_context = Some(task_group_ids);
                let delete_confirmation = self.fur_settings.show_delete_confirmation;

                return Command::perform(
                    async move {
                        if delete_confirmation {
                            if number_of_tasks > 1 {
                                Message::ShowAlert(FurAlert::DeleteGroupConfirmation)
                            } else {
                                Message::ShowAlert(FurAlert::DeleteTaskConfirmation)
                            }
                        } else {
                            Message::DeleteTasks
                        }
                    },
                    |msg| msg,
                );
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
            Message::EditShortcutPressed(shortcut) => {
                self.shortcut_to_edit = Some(ShortcutToEdit::new_from(&shortcut));
                self.inspector_view = Some(FurInspectorView::EditShortcut);
            }
            Message::EditShortcutTextChanged(new_value, property) => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    match property {
                        EditTaskProperty::Name => {
                            if new_value.contains('#')
                                || new_value.contains('@')
                                || new_value.contains('$')
                            {
                                shortcut_to_add.input_error(self.localization.get_message("name-cannot-contain", None));
                            } else {
                                shortcut_to_add.name = new_value;
                                shortcut_to_add.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Project => {
                            if new_value.contains('#')
                                || new_value.contains('@')
                                || new_value.contains('$')
                            {
                                shortcut_to_add.input_error(self.localization.get_message("project-cannot-contain", None));
                            } else {
                                shortcut_to_add.project = new_value;
                                shortcut_to_add.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Tags => {
                            if new_value.contains('@') || new_value.contains('$') {
                                shortcut_to_add.input_error(self.localization.get_message("tags-cannot-contain", None));
                            } else if !new_value.is_empty() && new_value.chars().next() != Some('#')
                            {
                                shortcut_to_add.input_error(self.localization.get_message("tags-must-start", None));
                            } else {
                                shortcut_to_add.tags = new_value;
                                shortcut_to_add.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Rate => {
                            let new_value_parsed = new_value.parse::<f32>();
                            if new_value.is_empty() {
                                shortcut_to_add.new_rate = String::new();
                            } else if new_value.contains('$') {
                                shortcut_to_add.input_error(self.localization.get_message("no-symbol-in-rate", None));
                            } else if new_value_parsed.is_ok()
                                && has_max_two_decimals(&new_value)
                                && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                            {
                                shortcut_to_add.new_rate = new_value;
                                shortcut_to_add.input_error(String::new());
                            } else {
                                shortcut_to_add.input_error(self.localization.get_message("rate-invalid", None));
                            }
                        }
                        _ => {}
                    }
                } else if let Some(shortcut_to_edit) = self.shortcut_to_edit.as_mut() {
                    match property {
                        EditTaskProperty::Name => {
                            if new_value.contains('#')
                                || new_value.contains('@')
                                || new_value.contains('$')
                            {
                                shortcut_to_edit.input_error(self.localization.get_message("name-cannot-contain", None))
                            } else {
                                shortcut_to_edit.new_name = new_value;
                                shortcut_to_edit.input_error(String::new())
                            }
                        }
                        EditTaskProperty::Project => {
                            if new_value.contains('#')
                                || new_value.contains('@')
                                || new_value.contains('$')
                            {
                                shortcut_to_edit.input_error(self.localization.get_message("project-cannot-contain", None));
                            } else {
                                shortcut_to_edit.new_project = new_value;
                                shortcut_to_edit.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Tags => {
                            if new_value.contains('@') || new_value.contains('$') {
                                shortcut_to_edit.input_error(self.localization.get_message("tags-cannot-contain", None));
                            } else if !new_value.is_empty() && new_value.chars().next() != Some('#')
                            {
                                shortcut_to_edit.input_error(self.localization.get_message("tags-must-start", None));
                            } else {
                                shortcut_to_edit.new_tags = new_value;
                                shortcut_to_edit.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Rate => {
                            let new_value_parsed = new_value.parse::<f32>();
                            if new_value.is_empty() {
                                shortcut_to_edit.new_rate = String::new();
                                shortcut_to_edit.input_error(String::new());
                            } else if new_value.contains('$') {
                                shortcut_to_edit.input_error(self.localization.get_message("no-symbol-in-rate", None));
                            } else if new_value_parsed.is_ok()
                                && has_max_two_decimals(&new_value)
                                && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                            {
                                shortcut_to_edit.new_rate = new_value;
                                shortcut_to_edit.input_error(String::new());
                            } else {
                                shortcut_to_edit.input_error(self.localization.get_message("rate-invalid", None));
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
                                            self.localization.get_message("name-cannot-contain", None);
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
                                            .input_error(self.localization.get_message("project-cannot-contain", None));
                                    } else {
                                        task_to_add.project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        task_to_add.input_error(self.localization.get_message("tags-cannot-contain", None));
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        task_to_add.input_error(self.localization.get_message("tags-must-start", None));
                                    } else {
                                        task_to_add.tags = new_value;
                                        task_to_add.input_error(String::new());
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        task_to_add.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        task_to_add.input_error(self.localization.get_message("no-symbol-in-rate", None));
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(&new_value)
                                        && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                    {
                                        task_to_add.new_rate = new_value;
                                        task_to_add.input_error(String::new());
                                    } else {
                                        task_to_add
                                            .input_error(self.localization.get_message("rate-invalid", None));
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
                                            .input_error(self.localization.get_message("name-cannot-contain", None));
                                    } else {
                                        task_to_edit.new_name = new_value;
                                        task_to_edit.input_error(String::new());
                                    }
                                }
                                EditTaskProperty::Project => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        task_to_edit
                                            .input_error(self.localization.get_message("project-cannot-contain", None));
                                    } else {
                                        task_to_edit.new_project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        task_to_edit.input_error(self.localization.get_message("tags-cannot-contain", None));
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        task_to_edit.input_error(self.localization.get_message("tags-must-start", None));
                                    } else {
                                        task_to_edit.new_tags = new_value;
                                        task_to_edit.input_error(String::new());
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        task_to_edit.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        task_to_edit.input_error(self.localization.get_message("no-symbol-in-rate", None));
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(&new_value)
                                        && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                    {
                                        task_to_edit.new_rate = new_value;
                                        task_to_edit.input_error(String::new());
                                    } else {
                                        task_to_edit
                                            .input_error(self.localization.get_message("rate-invalid", None));
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
                                            .input_error(self.localization.get_message("name-cannot-contain", None));
                                    } else {
                                        group_to_edit.new_name = new_value;
                                        group_to_edit.input_error(String::new());
                                    }
                                }
                                EditTaskProperty::Project => {
                                    if new_value.contains('#')
                                        || new_value.contains('@')
                                        || new_value.contains('$')
                                    {
                                        group_to_edit
                                            .input_error(self.localization.get_message("project-cannot-contain", None));
                                    } else {
                                        group_to_edit.new_project = new_value;
                                    }
                                }
                                EditTaskProperty::Tags => {
                                    if new_value.contains('@') || new_value.contains('$') {
                                        group_to_edit.input_error(self.localization.get_message("tags-cannot-contain", None));
                                    } else if !new_value.is_empty()
                                        && new_value.chars().next() != Some('#')
                                    {
                                        group_to_edit.input_error(self.localization.get_message("tags-must-start", None));
                                    } else {
                                        group_to_edit.new_tags = new_value;
                                        group_to_edit.input_error(String::new());
                                    }
                                }
                                EditTaskProperty::Rate => {
                                    let new_value_parsed = new_value.parse::<f32>();
                                    if new_value.is_empty() {
                                        group_to_edit.new_rate = String::new();
                                    } else if new_value.contains('$') {
                                        group_to_edit
                                            .input_error(self.localization.get_message("no-symbol-in-rate", None));
                                    } else if new_value_parsed.is_ok()
                                        && has_max_two_decimals(&new_value)
                                        && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                    {
                                        group_to_edit.new_rate = new_value;
                                        group_to_edit.input_error(String::new());
                                    } else {
                                        group_to_edit
                                            .input_error(self.localization.get_message("rate-invalid", None));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::EnterPressedInTaskInput => {
                if !self.task_input.is_empty() {
                    if !self.timer_is_running {
                        return Command::perform(async { Message::StartStopPressed }, |msg| msg);
                    }
                }
            }
            Message::ExportCsvPressed => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_error = Ok(String::new());
                let file_name = format!("furtherance-{}.csv", Local::now().format("%Y-%m-%d"));
                let selected_file = FileDialog::new()
                    .set_title("Save Furtherance CSV")
                    .add_filter("CSV", &["csv"])
                    .set_can_create_directories(true)
                    .set_file_name(file_name)
                    .save_file();

                if let Some(path) = selected_file {
                    match write_furtasks_to_csv(path) {
                        Ok(_) => self.settings_csv_message = Ok("CSV file saved.".to_string()),
                        Err(e) => {
                            eprintln!("Error writing data to CSV: {}", e);
                            self.settings_csv_message = Err("Error writing data to CSV.".into());
                        }
                    }
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
            }
            Message::ImportCsvPressed => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_error = Ok(String::new());
                let selected_file = FileDialog::new()
                    .set_title("Open Furtherance CSV")
                    .add_filter("CSV", &["csv"])
                    .set_can_create_directories(false)
                    .pick_file();
                if let Some(path) = selected_file {
                    if let Ok(mut file) = File::open(path) {
                        match verify_csv(&file) {
                            Ok(_) => {
                                import_csv_to_database(&mut file);
                                self.settings_csv_message = Ok("CSV imported successfully".into());
                            }
                            Err(e) => {
                                eprintln!("Invalid CSV file: {}", e);
                                self.settings_csv_message = Err("Invalid CSV file".into());
                            }
                        }
                    }
                }
            }
            Message::MidnightReached => {
                self.task_history = get_task_history(self.fur_settings.days_to_show);
            }
            Message::NavigateTo(destination) => {
                if self.current_view != destination {
                    self.inspector_view = None;
                    self.current_view = destination;
                    if destination == FurView::Report {
                        self.report.update_tasks_in_range();
                    }
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
                    + TimeDelta::minutes(self.fur_settings.pomodoro_break_length);
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
                self.inspector_view = None;
                self.task_to_add = None;
                self.task_to_edit = None;
                self.current_view = FurView::Timer;
                return Command::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::ReportTabSelected(new_tab) => self.report.active_tab = new_tab,
            Message::SaveGroupEdit => {
                if let Some(group_to_edit) = &self.group_to_edit {
                    let _ = db_update_group_of_tasks(group_to_edit);
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                }
            }
            Message::SaveShortcut => {
                if let Some(shortcut_to_add) = &self.shortcut_to_add {
                    let new_shortcut = FurShortcut {
                        id: 0,
                        name: shortcut_to_add.name.trim().to_string(),
                        tags: shortcut_to_add.tags.trim().to_string(),
                        project: shortcut_to_add.project.trim().to_string(),
                        rate: shortcut_to_add
                            .new_rate
                            .trim()
                            .parse::<f32>()
                            .unwrap_or(0.0),
                        currency: String::new(),
                        color_hex: shortcut_to_add.color.to_hex(),
                    };
                    match db_shortcut_exists(&new_shortcut) {
                        Ok(exists) => {
                            if exists {
                                self.displayed_alert = Some(FurAlert::ShortcutExists);
                            } else {
                                if let Err(e) = db_write_shortcut(new_shortcut) {
                                    eprintln!("Failed to write shortcut to database: {}", e);
                                }
                                self.inspector_view = None;
                                self.shortcut_to_add = None;
                                match db_retrieve_shortcuts() {
                                    Ok(shortcuts) => self.shortcuts = shortcuts,
                                    Err(e) => eprintln!(
                                        "Failed to retrieve shortcuts from database: {}",
                                        e
                                    ),
                                };
                            }
                        }
                        Err(e) => eprintln!("Failed to check if shortcut exists: {}", e),
                    }
                } else if let Some(shortcut_to_edit) = &self.shortcut_to_edit {
                    if let Err(e) = db_update_shortcut(FurShortcut {
                        id: shortcut_to_edit.id,
                        name: shortcut_to_edit.new_name.trim().to_string(),
                        tags: shortcut_to_edit.new_tags.trim().to_string(),
                        project: shortcut_to_edit.new_project.trim().to_string(),
                        rate: shortcut_to_edit
                            .new_rate
                            .trim()
                            .parse::<f32>()
                            .unwrap_or(0.0),
                        currency: String::new(),
                        color_hex: shortcut_to_edit.new_color.to_hex(),
                    }) {
                        eprintln!("Failed to update shortcut in database: {}", e);
                    }
                    self.inspector_view = None;
                    self.shortcut_to_edit = None;
                    match db_retrieve_shortcuts() {
                        Ok(shortcuts) => self.shortcuts = shortcuts,
                        Err(e) => eprintln!("Failed to retrieve shortcuts from database: {}", e),
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
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
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
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                }
            }
            Message::SettingsChangeDatabaseLocationPressed(new_or_open) => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_error = Ok(String::new());
                let path = Path::new(&self.fur_settings.database_url);
                let starting_dialog = FileDialog::new()
                    .set_directory(&path)
                    .add_filter("SQLite files", ALLOWED_DB_EXTENSIONS)
                    .set_can_create_directories(true);

                let selected_file = match new_or_open {
                    ChangeDB::New => starting_dialog
                        .set_file_name("furtherance.db")
                        .set_title("New Furtherance Database")
                        .save_file(),
                    ChangeDB::Open => starting_dialog
                        .set_title("Open Furtherance Database")
                        .pick_file(),
                };

                let mut is_old_db = false;

                if let Some(file) = selected_file {
                    self.settings_database_error = Ok(String::new());

                    if file.exists() {
                        match db_is_valid_v3(file.as_path()) {
                            Err(e) => {
                                eprintln!("Invalid database: {}", e);
                                self.settings_database_error = Err("Invalid database.".into());
                            }
                            Ok(is_valid_v3) => {
                                if !is_valid_v3 {
                                    match db_is_valid_v1(file.as_path()) {
                                        Ok(is_valid_v2) => {
                                            if is_valid_v2 {
                                                is_old_db = true
                                            } else {
                                                self.settings_database_error =
                                                    Err("Invalid database.".into());
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Invalid v1 database: {}", e);
                                            self.settings_database_error =
                                                Err("Invalid database.".into());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if self.settings_database_error.is_ok() {
                        // Valid file or not yet a file
                        if let Some(file_str) = file.to_str() {
                            if let Ok(_) = self.fur_settings.change_db_url(file_str) {
                                match db_init() {
                                    Ok(_) => {
                                        if is_old_db {
                                            if let Err(e) = db_upgrade_old() {
                                                eprintln!("Error upgrading legacy database: {}", e);
                                                self.settings_database_error =
                                                    Err("Error upgrading legacy database.".into());
                                                return Command::none();
                                            }
                                        }
                                        self.task_history =
                                            get_task_history(self.fur_settings.days_to_show);
                                        self.settings_database_error = Ok(match new_or_open {
                                            ChangeDB::Open => "Database loaded.".to_string(),
                                            ChangeDB::New => "Database created.".to_string(),
                                        });
                                    }
                                    Err(e) => {
                                        eprintln!("Error accessing new database: {}", e);
                                        self.settings_database_error =
                                            Err("Error accessing new database.".into());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Message::SettingsDatabaseLocationInputChanged(_) => {}
            Message::SettingsDaysToShowChanged(new_days) => {
                if new_days >= 1 {
                    match self.fur_settings.change_days_to_show(&new_days) {
                        Ok(_) => self.task_history = get_task_history(new_days),
                        Err(e) => eprintln!("Failed to change days_to_show in settings: {}", e),
                    }
                }
            }
            Message::SettingsDefaultViewSelected(selected_view) => {
                if let Err(e) = self.fur_settings.change_default_view(&selected_view) {
                    eprintln!("Failed to change default_view in settings: {}", e);
                }
            }
            Message::SettingsDeleteConfirmationToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_delete_confirmation(&new_value)
                {
                    eprintln!(
                        "Failed to change show_delete_confirmation in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsDynamicTotalToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_dynamic_total(&new_value) {
                    eprintln!("Failed to change dynamic_total in settings: {}", e);
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
            Message::SettingsShowChartAverageEarningsToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_average_earnings(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_average_earnings in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartAverageTimeToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_chart_average_time(&new_value) {
                    eprintln!(
                        "Failed to change show_chart_average_time in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartBreakdownBySelectionToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_breakdown_by_selection(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_breakdown_by_selection in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartEarningsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_chart_earnings(&new_value) {
                    eprintln!("Failed to change show_chart_earnings in settings: {}", e);
                }
            }
            Message::SettingsShowChartSelectionEarningsToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_selection_earnings(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_selection_earnings in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartSelectionTimeToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_selection_time(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_selection_time in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartTimeRecordedToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_time_recorded(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_time_recorded in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartTotalEarningsBoxToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_total_earnings_box(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_total_earnings_box in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsShowChartTotalTimeBoxToggled(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_show_chart_total_time_box(&new_value)
                {
                    eprintln!(
                        "Failed to change show_chart_total_time_box in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsTabSelected(new_tab) => self.settings_active_tab = new_tab,
            Message::ShortcutPressed(shortcut_task_input) => {
                self.task_input = shortcut_task_input;
                self.inspector_view = None;
                self.shortcut_to_add = None;
                self.shortcut_to_edit = None;
                self.current_view = FurView::Timer;
                return Command::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::ShowAlert(alert_to_show) => self.displayed_alert = Some(alert_to_show),
            Message::SettingsShowProjectToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_project(&new_value) {
                    eprintln!("Failed to change show_project in settings: {}", e);
                }
            }
            Message::SettingsShowTagsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_tags(&new_value) {
                    eprintln!("Failed to change show_tags in settings: {}", e);
                }
            }
            Message::SettingsShowEarningsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_earnings(&new_value) {
                    eprintln!("Failed to change show_earnings in settings: {}", e);
                }
            }
            Message::SettingsShowSecondsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_seconds(&new_value) {
                    eprintln!("Failed to change show_seconds in settings: {}", e);
                }
            }
            Message::SettingsShowDailyTimeTotalToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_daily_time_total(&new_value) {
                    eprintln!("Failed to change show_daily_time_total in settings: {}", e);
                }
            }
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
                                show_notification(NotificationType::BreakOver);
                                self.displayed_alert = Some(FurAlert::PomodoroBreakOver);
                            } else {
                                show_notification(NotificationType::PomodoroOver);
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
                                - TimeDelta::seconds(self.fur_settings.chosen_idle_time * 60);
                        } else if idle_time < self.fur_settings.chosen_idle_time * 60
                            && self.idle.reached
                            && !self.idle.notified
                        {
                            // User is back - show idle message
                            self.idle.notified = true;
                            show_notification(NotificationType::Idle);
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
            Message::SubmitEndDate(new_date) => self.report.set_date_range_end(new_date),
            Message::SubmitShortcutColor(new_color) => {
                if let Some(shortcut_to_add) = self.shortcut_to_add.as_mut() {
                    shortcut_to_add.color = new_color;
                    shortcut_to_add.show_color_picker = false;
                } else if let Some(shortcut_to_edit) = self.shortcut_to_edit.as_mut() {
                    shortcut_to_edit.new_color = new_color;
                    shortcut_to_edit.show_color_picker = false;
                }
            }
            Message::SubmitStartDate(new_date) => self.report.set_date_range_start(new_date),
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
                nav_button(
                    "Shortcuts",
                    FurView::Shortcuts,
                    self.current_view == FurView::Shortcuts
                ),
                nav_button("Timer", FurView::Timer, self.current_view == FurView::Timer),
                nav_button(
                    "History",
                    FurView::History,
                    self.current_view == FurView::History
                ),
                nav_button(
                    "Report",
                    FurView::Report,
                    self.current_view == FurView::Report
                ),
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
                nav_button(
                    "Settings",
                    FurView::Settings,
                    self.current_view == FurView::Settings
                ),
            ]
            .spacing(12)
            .align_items(Alignment::Start),
        )
        .width(175)
        .padding(10)
        .clip(true)
        .style(style::gray_background);

        // MARK: Shortcuts
        let mut shortcuts_row = FlowRow::new().spacing(20.0);
        for shortcut in &self.shortcuts {
            shortcuts_row = shortcuts_row.push(shortcut_button(shortcut, self.timer_is_running));
        }
        let shortcuts_view = column![
            row![
                horizontal_space(),
                button(bootstrap::icon_to_text(bootstrap::Bootstrap::PlusLg))
                    .on_press(Message::AddNewShortcutPressed)
                    .style(theme::Button::Text),
            ]
            .padding([10, 20]),
            Scrollable::new(column![shortcuts_row].padding(20))
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
                        .on_submit(Message::EnterPressedInTaskInput)
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
        for (i, (date, task_groups)) in self.task_history.iter().rev().enumerate() {
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
            all_history_rows = all_history_rows.push(history_title_row(
                date,
                total_time,
                total_earnings,
                &self.fur_settings,
                if i == 0 {
                    Some((self.timer_is_running, &self.timer_text))
                } else {
                    None
                },
            ));
            for task_group in task_groups {
                all_history_rows = all_history_rows.push(history_group_row(
                    task_group,
                    self.timer_is_running,
                    &self.fur_settings,
                ))
            }
        }
        let history_view = column![Scrollable::new(all_history_rows)
            .width(Length::FillPortion(3)) // TODO: Adjust?
            .height(Length::Fill)];

        // MARK: REPORT
        let mut charts_column = Column::new().align_items(Alignment::Center);

        let mut timer_earnings_boxes_widgets: Vec<Element<'_, Message, Theme, Renderer>> =
            Vec::new();
        if self.fur_settings.show_chart_total_time_box && self.report.total_time > 0 {
            timer_earnings_boxes_widgets.push(
                column![
                    text(seconds_to_formatted_duration(self.report.total_time, true)).size(50),
                    text("Total Time"),
                ]
                .align_items(Alignment::Center)
                .into(),
            );
        }
        if self.fur_settings.show_chart_total_earnings_box && self.report.total_earned > 0.0 {
            timer_earnings_boxes_widgets.push(
                column![
                    text(format!("${:.2}", self.report.total_earned)).size(50),
                    text("Earned"),
                ]
                .align_items(Alignment::Center)
                .into(),
            );
        }
        if !timer_earnings_boxes_widgets.is_empty() {
            // If both boxes are present, place a spacer between them
            if timer_earnings_boxes_widgets.len() == 2 {
                timer_earnings_boxes_widgets
                    .insert(1, horizontal_space().width(Length::Fill).into());
            }
            // Then place the bookend spacers
            timer_earnings_boxes_widgets.insert(0, horizontal_space().width(Length::Fill).into());
            timer_earnings_boxes_widgets.push(horizontal_space().width(Length::Fill).into());

            charts_column = charts_column
                .push(Row::with_children(timer_earnings_boxes_widgets).padding([0, 0, 10, 0]));
        }

        if self.fur_settings.show_chart_time_recorded {
            charts_column = charts_column.push(self.report.time_recorded_chart.view());
        }
        if self.fur_settings.show_chart_earnings && self.report.total_earned > 0.0 {
            charts_column = charts_column.push(self.report.earnings_chart.view());
        }
        if self.fur_settings.show_chart_average_time {
            charts_column = charts_column.push(self.report.average_time_chart.view());
        }
        if self.fur_settings.show_chart_average_earnings && self.report.total_earned > 0.0 {
            charts_column = charts_column.push(self.report.average_earnings_chart.view());
        }

        // Breakdown by Selection Picker & Charts
        let mut charts_breakdown_by_selection_column = Column::new().align_items(Alignment::Center);
        if !self.report.tasks_in_range.is_empty()
            && self.fur_settings.show_chart_breakdown_by_selection
        {
            charts_breakdown_by_selection_column =
                charts_breakdown_by_selection_column.push(text("Breakdown By Selection").size(40));
            charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                row![
                    pick_list(
                        &FurTaskProperty::ALL[..],
                        self.report.picked_task_property_key.clone(),
                        Message::ChartTaskPropertyKeySelected,
                    )
                    .width(Length::Fill),
                    pick_list(
                        &self.report.task_property_value_keys[..],
                        self.report.picked_task_property_value.clone(),
                        Message::ChartTaskPropertyValueSelected,
                    )
                    .width(Length::Fill),
                ]
                .spacing(10)
                .width(Length::Fill),
            );
            charts_breakdown_by_selection_column =
                charts_breakdown_by_selection_column.push(horizontal_rule(20));
            if self.fur_settings.show_chart_selection_time {
                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column
                    .push(self.report.selection_time_recorded_chart.view());
            }
            if self.fur_settings.show_chart_seleciton_earnings {
                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column
                    .push(self.report.selection_earnings_recorded_chart.view());
            }
        };

        let charts_view = column![
            column![
                pick_list(
                    &FurDateRange::ALL[..],
                    self.report.picked_date_range.clone(),
                    Message::DateRangeSelected,
                )
                .width(Length::Fill),
                if self.report.picked_date_range == Some(FurDateRange::Range) {
                    row![
                        horizontal_space().width(Length::Fill),
                        date_picker(
                            self.report.show_start_date_picker,
                            self.report.picked_start_date,
                            button(text(self.report.picked_start_date.to_string()))
                                .on_press(Message::ChooseStartDate),
                            Message::CancelStartDate,
                            Message::SubmitStartDate,
                        ),
                        column![text("to")
                            .vertical_alignment(alignment::Vertical::Center)
                            .height(Length::Fill),]
                        .height(30),
                        date_picker(
                            self.report.show_end_date_picker,
                            self.report.picked_end_date,
                            button(text(self.report.picked_end_date.to_string()))
                                .on_press(Message::ChooseEndDate),
                            Message::CancelEndDate,
                            Message::SubmitEndDate,
                        ),
                        horizontal_space().width(Length::Fill),
                    ]
                    .spacing(30)
                    .padding([20, 0, 0, 0])
                } else {
                    row![]
                },
                vertical_space().height(Length::Fixed(20.0)),
                horizontal_rule(1),
            ]
            .padding([20, 20, 10, 20]),
            Scrollable::new(
                column![charts_column, charts_breakdown_by_selection_column]
                    .align_items(Alignment::Center)
                    .padding([0, 20, 20, 20])
            ),
        ];

        // TODO: Change to tabbed report view once iced has lists
        // let report_view: Column<'_, Message, Theme, Renderer> =
        //     column![Tabs::new(Message::ReportTabSelected)
        //         .tab_icon_position(iced_aw::tabs::Position::Top)
        //         .push(
        //             TabId::Charts,
        //             TabLabel::IconText(
        //                 bootstrap::icon_to_char(Bootstrap::GraphUp),
        //                 "Charts".to_string()
        //             ),
        //             charts_view,
        //         )
        //         .push(
        //             TabId::List,
        //             TabLabel::IconText(
        //                 bootstrap::icon_to_char(Bootstrap::ListNested),
        //                 "List".to_string()
        //             ),
        //             Scrollable::new(column![].padding(10)),
        //         )
        //         .set_active_tab(&self.report.active_tab)
        //         .tab_bar_position(TabBarPosition::Top)];

        // MARK: SETTINGS
        let mut database_location_col = column![
            text("Database location"),
            text_input(
                &self.fur_settings.database_url,
                &self.fur_settings.database_url,
            ),
            row![
                button("Create New").on_press(Message::SettingsChangeDatabaseLocationPressed(
                    ChangeDB::New
                )),
                button("Open Existing").on_press(Message::SettingsChangeDatabaseLocationPressed(
                    ChangeDB::Open
                )),
            ]
            .spacing(10),
        ]
        .spacing(10);
        database_location_col =
            database_location_col.push_maybe(match &self.settings_database_error {
                Ok(msg) => {
                    if msg.is_empty() {
                        None
                    } else {
                        Some(text(msg).style(theme::Text::Color(Color::from_rgb(0.0, 255.0, 0.0))))
                    }
                }
                Err(e) => Some(text(e).style(theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0)))),
            });

        let mut csv_col = column![row![
            button("Export CSV").on_press(Message::ExportCsvPressed),
            button("Import CSV").on_press(Message::ImportCsvPressed)
        ]
        .spacing(10),]
        .spacing(10);
        csv_col = csv_col.push_maybe(match &self.settings_csv_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(theme::Text::Color(Color::from_rgb(0.0, 255.0, 0.0))))
                }
            }
            Err(e) => Some(text(e).style(theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0)))),
        });

        let mut backup_col =
            column![button("Backup Database").on_press(Message::BackupDatabase)].spacing(10);
        backup_col = backup_col.push_maybe(match &self.settings_backup_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(theme::Text::Color(Color::from_rgb(0.0, 255.0, 0.0))))
                }
            }
            Err(e) => Some(text(e).style(theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0)))),
        });

        let settings_view: Column<'_, Message, Theme, Renderer> =
            column![Tabs::new(Message::SettingsTabSelected)
                .tab_icon_position(iced_aw::tabs::Position::Top)
                .push(
                    TabId::General,
                    TabLabel::IconText(
                        bootstrap::icon_to_char(Bootstrap::GearFill),
                        "General".to_string()
                    ),
                    Scrollable::new(
                        column![
                            settings_heading("Interface"),
                            row![
                                text("Default view"),
                                pick_list(
                                    &FurView::ALL[..],
                                    Some(self.fur_settings.default_view),
                                    Message::SettingsDefaultViewSelected,
                                ),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            row![
                                text("Show delete confirmation"),
                                toggler(
                                    String::new(),
                                    self.fur_settings.show_delete_confirmation,
                                    Message::SettingsDeleteConfirmationToggled
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            settings_heading("Task History"),
                            row![
                                text("Show project"),
                                toggler(
                                    String::new(),
                                    self.fur_settings.show_project,
                                    Message::SettingsShowProjectToggled
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            row![
                                text("Show tags"),
                                toggler(
                                    String::new(),
                                    self.fur_settings.show_tags,
                                    Message::SettingsShowTagsToggled
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            row![
                                text("Show earnings"),
                                toggler(
                                    String::new(),
                                    self.fur_settings.show_earnings,
                                    Message::SettingsShowEarningsToggled
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            row![
                                text("Show seconds"),
                                toggler(
                                    String::new(),
                                    self.fur_settings.show_seconds,
                                    Message::SettingsShowSecondsToggled
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            row![
                                text("Show daily time total"),
                                toggler(
                                    String::new(),
                                    self.fur_settings.show_daily_time_total,
                                    Message::SettingsShowDailyTimeTotalToggled
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                        ]
                        .spacing(SETTINGS_SPACING)
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
                            settings_heading("Idle"),
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
                            settings_heading("Task History"),
                            row![
                                column![
                                    text("Dynamic total"),
                                    text("Today's total time ticks up with the timer").size(12),
                                ],
                                toggler(
                                    String::new(),
                                    self.fur_settings.dynamic_total,
                                    Message::SettingsDynamicTotalToggled
                                )
                                .width(Length::Shrink)
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                            row![
                                text("Days to show"),
                                number_input(
                                    self.fur_settings.days_to_show,
                                    365, // TODO: This will accept a range in a future version of iced_aw (make 1..365)
                                    Message::SettingsDaysToShowChanged
                                )
                                .width(Length::Shrink),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                        ]
                        .spacing(SETTINGS_SPACING)
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
                            settings_heading("Pomodoro timer"),
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
                            settings_heading("Extended break"),
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
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                .push(
                    TabId::Report,
                    TabLabel::IconText(
                        bootstrap::icon_to_char(Bootstrap::GraphUp),
                        "Report".to_string()
                    ),
                    Scrollable::new(
                        column![
                            settings_heading("Toggle charts"),
                            checkbox(
                                "Total time box",
                                self.fur_settings.show_chart_total_time_box
                            )
                            .on_toggle(Message::SettingsShowChartTotalTimeBoxToggled),
                            checkbox(
                                "Total earnings box",
                                self.fur_settings.show_chart_total_earnings_box
                            )
                            .on_toggle(Message::SettingsShowChartTotalEarningsBoxToggled),
                            checkbox("Time recorded", self.fur_settings.show_chart_time_recorded)
                                .on_toggle(Message::SettingsShowChartTimeRecordedToggled),
                            checkbox("Earnings", self.fur_settings.show_chart_earnings)
                                .on_toggle(Message::SettingsShowChartEarningsToggled),
                            checkbox(
                                "Average time per task",
                                self.fur_settings.show_chart_average_time
                            )
                            .on_toggle(Message::SettingsShowChartAverageTimeToggled),
                            checkbox(
                                "Average earnings per task",
                                self.fur_settings.show_chart_average_earnings
                            )
                            .on_toggle(Message::SettingsShowChartAverageEarningsToggled),
                            checkbox(
                                "Breakdown by selection section",
                                self.fur_settings.show_chart_breakdown_by_selection
                            )
                            .on_toggle(Message::SettingsShowChartBreakdownBySelectionToggled),
                            checkbox(
                                "Time recorded for selection",
                                self.fur_settings.show_chart_selection_time
                            )
                            .on_toggle_maybe(
                                if self.fur_settings.show_chart_breakdown_by_selection {
                                    Some(Message::SettingsShowChartSelectionTimeToggled)
                                } else {
                                    None
                                }
                            ),
                            checkbox(
                                "Earnings for selection",
                                self.fur_settings.show_chart_seleciton_earnings
                            )
                            .on_toggle_maybe(
                                if self.fur_settings.show_chart_breakdown_by_selection {
                                    Some(Message::SettingsShowChartSelectionEarningsToggled)
                                } else {
                                    None
                                }
                            ),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                // MARK: SETTINGS DATA TAB
                .push(
                    TabId::Data,
                    TabLabel::IconText(
                        bootstrap::icon_to_char(Bootstrap::FloppyFill),
                        "Data".to_string()
                    ),
                    Scrollable::new(
                        column![
                            settings_heading("Local Database"),
                            database_location_col,
                            settings_heading("CSV"),
                            csv_col,
                            settings_heading("Backup"),
                            backup_col,
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
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
            // Add shortcut
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
                    text(&shortcut_to_add.invalid_input_error_message)
                        .style(theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0))),
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
            // MARK: Edit Shortcut
            Some(FurInspectorView::EditShortcut) => match &self.shortcut_to_edit {
                Some(shortcut_to_edit) => column![
                    text("Edit Shortcut").size(24),
                    text_input("Task name", &shortcut_to_edit.new_name)
                        .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Name)),
                    text_input("Project", &shortcut_to_edit.new_project).on_input(|s| {
                        Message::EditShortcutTextChanged(s, EditTaskProperty::Project)
                    }),
                    text_input("#tags", &shortcut_to_edit.new_tags)
                        .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Tags)),
                    row![
                        text("$"),
                        text_input("0.00", &shortcut_to_edit.new_rate).on_input(|s| {
                            Message::EditShortcutTextChanged(s, EditTaskProperty::Rate)
                        }),
                        text("/hr"),
                    ]
                    .spacing(3)
                    .align_items(Alignment::Center),
                    color_picker(
                        shortcut_to_edit.show_color_picker,
                        shortcut_to_edit.new_color,
                        button(
                            text("Color")
                                .style(if is_dark_color(shortcut_to_edit.new_color.to_srgb()) {
                                    Color::WHITE
                                } else {
                                    Color::BLACK
                                })
                                .width(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Center)
                        )
                        .on_press(Message::ChooseShortcutColor)
                        .width(Length::Fill)
                        .style(style::custom_button_style(
                            shortcut_to_edit.new_color.to_srgb(),
                        )),
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
                            .on_press_maybe(
                                if shortcut_to_edit.new_name.trim().is_empty()
                                    || !shortcut_to_edit.is_changed()
                                {
                                    None
                                } else {
                                    Some(Message::SaveShortcut)
                                }
                            )
                            .width(Length::Fill),
                    ]
                    .padding([20, 0, 0, 0])
                    .spacing(10),
                    text(&shortcut_to_edit.invalid_input_error_message)
                        .style(theme::Text::Color(Color::from_rgb(255.0, 0.0, 0.0))),
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
                            .on_press(if self.fur_settings.show_delete_confirmation {
                                Message::ShowAlert(FurAlert::DeleteTaskConfirmation)
                            } else {
                                Message::DeleteTasks
                            })
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
                                                        task.total_time_in_seconds(),
                                                        self.fur_settings.show_seconds
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
                                .on_press(if self.fur_settings.show_delete_confirmation {
                                    Message::ShowAlert(FurAlert::DeleteGroupConfirmation)
                                } else {
                                    Message::DeleteTasks
                                })
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
                FurView::Report => charts_view,
                FurView::Settings => settings_view,
            },
            row![vertical_rule(1), inspector].width(if self.inspector_view.is_some() {
                260
            } else {
                0
            }),
        ];

        let overlay: Option<Card<'_, Message, Theme, Renderer>> = if self.displayed_alert.is_some()
        {
            let alert_text: String;
            let alert_description: &str;
            let close_button: Option<Button<'_, Message, Theme, Renderer>>;
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
                FurAlert::DeleteShortcutConfirmation => {
                    alert_text = "Delete shortcut?".to_string();
                    alert_description = "Are you sure you want to delete this shortcut?";
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
                        .on_press(Message::DeleteShortcut)
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
                FurAlert::ShortcutExists => {
                    alert_text = "Shortcut exists".to_string();
                    alert_description = "A shortcut for that task already exists.";
                    close_button = Some(
                        button(
                            text("OK")
                                .horizontal_alignment(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
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

fn nav_button<'a>(text: &'a str, destination: FurView, active: bool) -> Button<'a, Message> {
    button(text)
        .padding([5, 15])
        .on_press(Message::NavigateTo(destination))
        .width(Length::Fill)
        .style(if active {
            style::active_nav_menu_button_style()
        } else {
            style::inactive_nav_menu_button_style()
        })
}

fn history_group_row<'a>(
    task_group: &FurTaskGroup,
    timer_is_running: bool,
    settings: &FurSettings,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'static>, Message> {
    let mut task_details_column: Column<'_, Message, Theme, Renderer> =
        column![text(&task_group.name).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),]
        .width(Length::FillPortion(6));
    if settings.show_project && !task_group.project.is_empty() {
        task_details_column = task_details_column.push(text(format!("@{}", task_group.project)));
    }
    if settings.show_tags && !task_group.tags.is_empty() {
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

    let total_time_str =
        seconds_to_formatted_duration(task_group.total_time, settings.show_seconds);
    let mut totals_column: Column<'_, Message, Theme, Renderer> = column![text(total_time_str)
        .font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })]
    .align_items(Alignment::End);

    if settings.show_earnings && task_group.rate > 0.0 {
        let total_earnings = task_group.rate * (task_group.total_time as f32 / 3600.0);
        totals_column = totals_column.push(text(&format!("${:.2}", total_earnings)));
    }

    let task_group_string = task_group.to_string();

    task_row = task_row.push(task_details_column);
    task_row = task_row.push(horizontal_space().width(Length::Fill));
    task_row = task_row.push(totals_column);
    task_row = task_row.push(
        button(bootstrap::icon_to_text(bootstrap::Bootstrap::ArrowRepeat))
            .on_press_maybe(if timer_is_running {
                None
            } else {
                Some(Message::RepeatLastTaskPressed(task_group_string.clone()))
            })
            .style(theme::Button::Text),
    );

    let history_row_button = button(
        Container::new(task_row)
            .padding([10, 15, 10, 15])
            .width(Length::Fill)
            .style(style::task_row),
    )
    .on_press(Message::EditGroup(task_group.clone()))
    .style(theme::Button::Text);

    let task_group_ids = task_group.all_task_ids();
    let task_group_clone = task_group.clone();
    ContextMenu::new(
        history_row_button,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button("Repeat")
                    .on_press(Message::RepeatLastTaskPressed(task_group_string.clone()))
                    .style(style::context_menu_button_style())
                    .width(Length::Fill),
                iced::widget::button("Edit")
                    .on_press(Message::EditGroup(task_group_clone.clone()))
                    .style(style::context_menu_button_style())
                    .width(Length::Fill),
                iced::widget::button("Create shortcut")
                    .on_press(Message::CreateShortcutFromTaskGroup(
                        task_group_clone.clone(),
                    ))
                    .style(style::context_menu_button_style())
                    .width(Length::Fill),
                iced::widget::button("Delete")
                    .on_press(Message::DeleteTasksFromContext(task_group_ids.clone()))
                    .style(style::context_menu_button_style())
                    .width(Length::Fill),
            ])
            .max_width(150)
            .into()
        }),
    )
}

fn history_title_row<'a>(
    date: &NaiveDate,
    total_time: i64,
    total_earnings: f32,
    settings: &FurSettings,
    running_timer: Option<(bool, &str)>,
) -> Row<'a, Message> {
    let mut total_time_column = column![].align_items(Alignment::End);

    if settings.show_daily_time_total {
        if let Some((running, timer_text)) = running_timer {
            if running {
                let total_time_str = seconds_to_formatted_duration(
                    combine_timer_with_seconds(timer_text, total_time),
                    settings.show_seconds,
                );
                total_time_column = total_time_column.push(text(total_time_str).font(font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }));
            }
        } else {
            let total_time_str = seconds_to_formatted_duration(total_time, settings.show_seconds);
            total_time_column = total_time_column.push(text(total_time_str).font(font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }));
        }
    }

    if settings.show_earnings && total_earnings > 0.0 {
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
    let yesterday = today - TimeDelta::days(1);
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

fn get_task_history(limit: i64) -> BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> {
    let mut grouped_tasks_by_date: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> = BTreeMap::new();

    match db_retrieve_tasks_with_day_limit(limit, SortBy::StopTime, SortOrder::Descending) {
        Ok(all_tasks) => {
            let tasks_by_date = group_tasks_by_date(all_tasks);

            for (date, tasks) in tasks_by_date {
                let mut all_groups: Vec<FurTaskGroup> = vec![];
                for task in tasks {
                    if let Some(matching_group) =
                        all_groups.iter_mut().find(|x| x.is_equal_to(&task))
                    {
                        matching_group.add(task);
                    } else {
                        all_groups.push(FurTaskGroup::new_from(task));
                    }
                }
                grouped_tasks_by_date.insert(date, all_groups);
            }
        }
        Err(e) => {
            eprintln!("Error retrieving tasks from database: {}", e);
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

fn shortcut_button_content<'a>(
    shortcut: &FurShortcut,
    text_color: Color,
) -> Column<'a, Message, Theme, Renderer> {
    let mut shortcut_text_column = column![text(shortcut.name.clone())
        .font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .style(text_color)]
    .spacing(5);

    if !shortcut.project.is_empty() {
        shortcut_text_column = shortcut_text_column
            .push(text(format!("@{}", shortcut.project.clone())).style(text_color));
    }
    if !shortcut.tags.is_empty() {
        shortcut_text_column =
            shortcut_text_column.push(text(shortcut.tags.clone()).style(text_color));
    }
    if shortcut.rate > 0.0 {
        shortcut_text_column = shortcut_text_column.push(vertical_space());
        shortcut_text_column = shortcut_text_column.push(row![
            horizontal_space(),
            text(format!("${:.2}", shortcut.rate)).style(text_color)
        ]);
    }

    shortcut_text_column
}

fn shortcut_button<'a>(
    shortcut: &FurShortcut,
    timer_is_running: bool,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'static>, Message> {
    let shortcut_color = match Srgb::from_hex(&shortcut.color_hex) {
        Ok(color) => color,
        Err(_) => Srgb::new(0.694, 0.475, 0.945),
    };
    let text_color = if is_dark_color(shortcut_color) {
        Color::WHITE
    } else {
        Color::BLACK
    };

    let shortcut_button = button(shortcut_button_content(&shortcut, text_color))
        .width(200)
        .padding(10)
        .height(170)
        .on_press_maybe(if timer_is_running {
            None
        } else {
            Some(Message::ShortcutPressed(shortcut.to_string()))
        })
        .style(style::custom_button_style(shortcut_color));

    let shortcut_clone = shortcut.clone();

    ContextMenu::new(
        shortcut_button,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button("Edit")
                    .on_press(Message::EditShortcutPressed(shortcut_clone.clone()))
                    .style(style::context_menu_button_style())
                    .width(Length::Fill),
                iced::widget::button("Delete")
                    .on_press(Message::DeleteShortcutFromContext(
                        shortcut_clone.id.clone()
                    ))
                    .style(style::context_menu_button_style())
                    .width(Length::Fill),
            ])
            .max_width(150)
            .into()
        }),
    )
}

fn is_dark_color(color: Srgb) -> bool {
    color.relative_luminance().luma < 0.6
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
    state.task_history = get_task_history(state.fur_settings.days_to_show);
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
                    true,
                )
            } else {
                seconds_to_formatted_duration(state.fur_settings.pomodoro_break_length * 60, true)
            }
        } else if state.pomodoro.snoozed {
            seconds_to_formatted_duration(state.fur_settings.pomodoro_snooze_length * 60, true)
        } else {
            seconds_to_formatted_duration(state.fur_settings.pomodoro_length * 60, true)
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
                    + TimeDelta::minutes(state.fur_settings.pomodoro_extended_break_length)
            } else {
                state.timer_start_time
                    + TimeDelta::minutes(state.fur_settings.pomodoro_break_length)
            }
        } else {
            if state.pomodoro.snoozed {
                state.pomodoro.snoozed_at
                    + TimeDelta::minutes(state.fur_settings.pomodoro_snooze_length)
            } else {
                state.timer_start_time + TimeDelta::minutes(state.fur_settings.pomodoro_length)
            }
        };

        let seconds_until_end =
            (stop_time - state.timer_start_time).num_seconds() - seconds_elapsed;
        if seconds_until_end > 0 {
            seconds_to_formatted_duration(seconds_until_end, true)
        } else {
            "0:00:00".to_string()
        }
    } else {
        seconds_to_formatted_duration(seconds_elapsed, true)
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
        if let Some((_, first_gorup)) = state.task_history.last_key_value() {
            if let Some(last_run_task) = first_gorup.first() {
                let task_input_builder = last_run_task.to_string();
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
    let mut total_seconds: i64 = if let Some(groups) = state.task_history.get(&today) {
        groups.iter().map(|group| group.total_time).sum()
    } else {
        0
    };
    if state.fur_settings.dynamic_total && state.timer_is_running {
        total_seconds = combine_timer_with_seconds(&state.timer_text, total_seconds);
    }
    seconds_to_formatted_duration(total_seconds, state.fur_settings.show_seconds)
}

fn seconds_to_formatted_duration(total_seconds: i64, show_seconds: bool) -> String {
    if show_seconds {
        seconds_to_hms(total_seconds)
    } else {
        seconds_to_hm(total_seconds)
    }
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

fn show_notification(notification_type: NotificationType) {
    let heading: &str;
    let details: &str;
    match notification_type {
        NotificationType::PomodoroOver => {
            heading = "Time's up!";
            details = "It's time to take a break.";
        }
        NotificationType::BreakOver => {
            heading = "Break's over!";
            details = "Time to get back to work.";
        }
        NotificationType::Idle => {
            heading = "You've been idle.";
            details = "Open Furtherance to continue or discard the idle time.";
        }
    }
    // TODO: Enable later
    // Notification::new()
    //     .summary(heading)
    //     .body(details)
    //     .appname("furtherance")
    //     .timeout(Timeout::Milliseconds(6000))
    //     .show()
    //     .unwrap();
}

fn settings_heading(heading: &str) -> Column<'_, Message, Theme, Renderer> {
    column![
        text(heading).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Container::new(horizontal_rule(1)).max_width(200.0)
    ]
    .padding([15, 0, 5, 0])
}

fn combine_timer_with_seconds(timer: &str, seconds: i64) -> i64 {
    let timer_seconds = parse_timer_text_to_seconds(timer);
    timer_seconds + seconds
}

fn parse_timer_text_to_seconds(timer_text: &str) -> i64 {
    let parts: Vec<&str> = timer_text.split(':').collect();
    match parts.len() {
        3 => {
            let hours: i64 = parts[0].parse().unwrap_or(0);
            let minutes: i64 = parts[1].parse().unwrap_or(0);
            let seconds: i64 = parts[2].parse().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        }
        2 => {
            let minutes: i64 = parts[0].parse().unwrap_or(0);
            let seconds: i64 = parts[1].parse().unwrap_or(0);
            minutes * 60 + seconds
        }
        1 => parts[0].parse().unwrap_or(0),
        _ => 0,
    }
}

fn get_system_theme() -> Theme {
    match dark_light::detect() {
        dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
        dark_light::Mode::Dark => Theme::Dark,
    }
}

pub fn write_furtasks_to_csv(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(file) = std::fs::File::create(path) {
        if let Ok(tasks) = db_retrieve_all_tasks(SortBy::StartTime, SortOrder::Descending) {
            let mut csv_writer = Writer::from_writer(file);
            csv_writer.write_record(&[
                "Name",
                "Start Time",
                "Stop Time",
                "Tags",
                "Project",
                "Rate",
                "Currency",
                "Total Time",
                "Total Earnings",
            ])?;

            for task in tasks {
                csv_writer.write_record(&[
                    task.name.clone(),
                    task.start_time.to_rfc3339(),
                    task.stop_time.to_rfc3339(),
                    task.tags.clone(),
                    task.project.clone(),
                    task.rate.to_string(),
                    task.currency.clone(),
                    seconds_to_formatted_duration(task.total_time_in_seconds(), true),
                    format!("${:.2}", task.total_earnings()),
                ])?;
            }

            csv_writer.flush()?;
            Ok(())
        } else {
            Err("Failed to retrieve tasks from the database".into())
        }
    } else {
        Err("Failed to create the file".into())
    }
}

pub fn verify_csv(file: &std::fs::File) -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = Reader::from_reader(file);

    let v3_headers = vec![
        "Name",
        "Start Time",
        "Stop Time",
        "Tags",
        "Project",
        "Rate",
        "Currency",
        "Total Time",
        "Total Earnings",
    ];
    let v2_headers = vec![
        "Name",
        "Project",
        "Tags",
        "Rate",
        "Start Time",
        "Stop Time",
        "Total Seconds",
    ];
    let v1_headers = vec![
        "id",
        "task_name",
        "start_time",
        "stop_time",
        "tags",
        "seconds",
    ];

    if let Ok(headers) = rdr.headers() {
        if verify_headers(headers, &v3_headers).is_err() {
            if verify_headers(headers, &v2_headers).is_err() {
                verify_headers(headers, &v1_headers)?;
            }
        }
    } else {
        return Err("Failed to read the headers".into());
    }

    Ok(())
}

fn verify_headers(
    headers: &StringRecord,
    expected: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    for (i, expected_header) in expected.iter().enumerate() {
        match headers.get(i) {
            Some(header) if header == *expected_header => continue,
            Some(_) => {
                return Err(format!("Wrong column order.").into());
            }
            None => {
                return Err(format!("Missing column").into());
            }
        }
    }
    Ok(())
}

pub fn read_csv(file: &File) -> Result<Vec<FurTask>, Box<dyn std::error::Error>> {
    let mut rdr = ReaderBuilder::new().flexible(true).from_reader(file);
    let mut tasks = Vec::new();

    for result in rdr.records() {
        let record = result?;

        let task = match record.len() {
            9 => FurTask {
                // v3 - Iced
                id: 0,
                name: record.get(0).unwrap_or("").to_string(),
                start_time: record.get(1).unwrap_or("").parse().unwrap_or_default(),
                stop_time: record.get(2).unwrap_or("").parse().unwrap_or_default(),
                tags: record.get(3).unwrap_or("").trim().to_string(),
                project: record.get(4).unwrap_or("").trim().to_string(),
                rate: record.get(5).unwrap_or("0").trim().parse().unwrap_or(0.0),
                currency: record.get(6).unwrap_or("").trim().to_string(),
            },
            7 => FurTask {
                // v2 - macOS SwiftUI
                id: 0,
                name: record.get(0).unwrap_or("").to_string(),
                start_time: record.get(4).unwrap_or("").parse().unwrap_or_default(),
                stop_time: record.get(5).unwrap_or("").parse().unwrap_or_default(),
                tags: record.get(2).unwrap_or("").trim().to_string(),
                project: record.get(1).unwrap_or("").trim().to_string(),
                rate: record.get(3).unwrap_or("0").trim().parse().unwrap_or(0.0),
                currency: String::new(),
            },
            6 => FurTask {
                // v1 - GTK
                id: record.get(0).unwrap_or("0").parse().unwrap_or(0),
                name: record.get(1).unwrap_or("").to_string(),
                start_time: record.get(2).unwrap_or("").parse().unwrap_or_default(),
                stop_time: record.get(3).unwrap_or("").parse().unwrap_or_default(),
                tags: record.get(4).unwrap_or("").trim().to_string(),
                project: String::new(),
                rate: 0.0,
                currency: String::new(),
            },

            _ => return Err("Invalid CSV".into()),
        };

        if let Ok(exists) = db_task_exists(&task) {
            if !exists {
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

pub fn import_csv_to_database(file: &mut File) {
    // Seek back to the start of the file after verification
    if let Err(e) = file.seek(std::io::SeekFrom::Start(0)) {
        eprintln!("Failed to seek to start of file: {}", e);
        return;
    }

    match read_csv(file) {
        Ok(tasks_to_import) => {
            if let Err(e) = db_write_tasks(&tasks_to_import) {
                eprintln!("Failed to import tasks: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to read the CSV file: {}", e),
    }
}
