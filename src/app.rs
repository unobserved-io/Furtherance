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
    collections::{BTreeMap, HashMap},
    fs::File,
    io::Seek,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{
    autosave::{autosave_exists, delete_autosave, restore_autosave, write_autosave},
    constants::{
        ALLOWED_DB_EXTENSIONS, FURTHERANCE_VERSION, INSPECTOR_ALIGNMENT, INSPECTOR_PADDING,
        INSPECTOR_SPACING, INSPECTOR_WIDTH, OFFICIAL_SERVER, SETTINGS_MESSAGE_DURATION,
        SETTINGS_SPACING,
    },
    database::*,
    helpers::{
        color_utils::{FromHex, RandomColor, ToHex, ToSrgb},
        idle,
        midnight_subscription::MidnightSubscription,
    },
    localization::Localization,
    models::{
        fur_idle::FurIdle,
        fur_pomodoro::FurPomodoro,
        fur_report::FurReport,
        fur_settings::FurSettings,
        fur_shortcut::{EncryptedShortcut, FurShortcut},
        fur_task::{EncryptedTask, FurTask},
        fur_task_group::FurTaskGroup,
        fur_user::{FurUser, FurUserFields},
        group_to_edit::GroupToEdit,
        shortcut_to_add::ShortcutToAdd,
        shortcut_to_edit::ShortcutToEdit,
        task_to_add::TaskToAdd,
        task_to_edit::TaskToEdit,
    },
    server::{
        encryption::{self, decrypt_encryption_key, encrypt_encryption_key},
        login::{login, ApiError, LoginResponse},
        logout,
        sync::{sync_with_server, SyncResponse},
    },
    style::{self, FurTheme},
    view_enums::*,
};
use chrono::{offset::LocalResult, DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use chrono::{TimeDelta, TimeZone, Timelike};
use csv::{Reader, ReaderBuilder, StringRecord, Writer};
use fluent::FluentValue;
use iced::{
    advanced::subscription,
    alignment, font,
    keyboard::{self, key},
    widget::{
        self, button, center, checkbox, column, container, horizontal_rule, horizontal_space,
        opaque, pick_list, row, stack, text, text_input, toggler, vertical_rule, vertical_space,
        Button, Column, Container, Row, Scrollable,
    },
    Alignment, Color, Element, Length, Padding, Renderer, Subscription, Task, Theme,
};
use iced_aw::{
    color_picker, date_picker, number_input, time_picker, Card, ContextMenu, TabBarPosition,
    TabLabel, Tabs, TimePicker,
};
use iced_fonts::{bootstrap::icon_to_char, Bootstrap, BOOTSTRAP_FONT};
use itertools::Itertools;
use notify_rust::{
    Notification, 
    Timeout, 
};
use palette::color_difference::Wcag21RelativeContrast;
use palette::Srgb;
use regex::Regex;
use rfd::FileDialog;
use tokio::time;
use webbrowser;

#[cfg(target_os = "linux")]
use crate::helpers::wayland_idle;
#[cfg(target_os = "macos")]
use notify_rust::set_application;

pub struct Furtherance {
    current_view: FurView,
    delete_ids_from_context: Option<Vec<String>>,
    delete_shortcut_from_context: Option<String>,
    displayed_alert: Option<FurAlert>,
    displayed_task_start_time: time_picker::Time,
    fur_settings: FurSettings,
    fur_user: Option<FurUser>,
    fur_user_fields: FurUserFields,
    group_to_edit: Option<GroupToEdit>,
    idle: FurIdle,
    inspector_view: Option<FurInspectorView>,
    localization: Arc<Localization>,
    login_message: Result<String, Box<dyn std::error::Error>>,
    pomodoro: FurPomodoro,
    report: FurReport,
    settings_active_tab: TabId,
    settings_csv_message: Result<String, Box<dyn std::error::Error>>,
    settings_database_message: Result<String, Box<dyn std::error::Error>>,
    settings_more_message: Result<String, Box<dyn std::error::Error>>,
    settings_server_choice: Option<ServerChoices>,
    shortcuts: Vec<FurShortcut>,
    shortcut_to_add: Option<ShortcutToAdd>,
    shortcut_to_edit: Option<ShortcutToEdit>,
    show_timer_start_picker: bool,
    task_history: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>>,
    task_input: String,
    theme: FurTheme,
    timer_is_running: bool,
    timer_start_time: DateTime<Local>,
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
    ClearLoginMessage,
    CreateShortcutFromTaskGroup(FurTaskGroup),
    DeleteEverything,
    DateRangeSelected(FurDateRange),
    DeleteShortcut,
    DeleteShortcutFromContext(String),
    DeleteTasks,
    DeleteTasksFromContext(Vec<String>),
    EditGroup(FurTaskGroup),
    EditShortcutPressed(FurShortcut),
    EditShortcutTextChanged(String, EditTaskProperty),
    EditTask(FurTask),
    EditTaskTextChanged(String, EditTaskProperty),
    EnterPressedInTaskInput,
    EnterPressedInSyncFields,
    ExportCsvPressed,
    FontLoaded(Result<(), font::Error>),
    IdleDiscard,
    IdleReset,
    ImportCsvPressed,
    ImportOldMacDatabase,
    LearnAboutSync,
    MidnightReached,
    NavigateTo(FurView),
    NotifyOfSyncClose,
    OpenUrl(String),
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
    SettingsPomodoroNotificationAlarmSoundToggled(bool),
    SettingsPomodoroLengthChanged(i64),
    SettingsPomodoroSnoozeLengthChanged(i64),
    SettingsPomodoroToggled(bool),
    SettingsReminderIntervalChanged(u16),
    SettingsRemindersToggled(bool),
    ShowReminderNotification,
    SettingsServerChoiceSelected(ServerChoices),
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
    SettingsThemeSelected(FurDarkLight),
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
    SyncWithServer,
    SyncComplete((Result<SyncResponse, ApiError>, usize)),
    TabPressed { shift: bool },
    TaskInputChanged(String),
    ToggleGroupEditor,
    UserAutoLogoutComplete,
    UserEmailChanged(String),
    UserLoginPressed,
    UserLoginComplete(Result<LoginResponse, ApiError>),
    UserLogoutPressed,
    UserLogoutComplete,
    UserEncryptionKeyChanged(String),
    UserServerChanged(String),
}

impl Furtherance {
    pub fn new() -> (Self, iced::Task<Message>) {
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

        // Load user credentials from database
        let saved_user = match db_retrieve_credentials() {
            Ok(optional_user) => optional_user,
            Err(e) => {
                eprintln!("Error retrieving user credentials from database: {}", e);
                None
            }
        };

        // Set application identifier for notifications
        #[cfg(target_os = "macos")]
        if let Err(e) = set_application("io.unobserved.furtherance") {
            eprintln!(
                "Failed to set application identifier for notifications: {}",
                e
            );
        }

        let mut furtherance = Furtherance {
            current_view: settings.default_view,
            delete_ids_from_context: None,
            delete_shortcut_from_context: None,
            displayed_alert: None,
            displayed_task_start_time: time_picker::Time::now_hm(true),
            fur_settings: settings,
            fur_user: saved_user.clone(),
            fur_user_fields: match &saved_user {
                Some(user) => FurUserFields {
                    email: user.email.clone(),
                    encryption_key: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
                    server: user.server.clone(),
                },
                None => FurUserFields::default(),
            },
            group_to_edit: None,
            idle: FurIdle::new(),
            localization: Arc::new(Localization::new()),
            login_message: Ok(String::new()),
            pomodoro: FurPomodoro::new(),
            inspector_view: None,
            report: FurReport::new(),
            settings_active_tab: TabId::General,
            settings_csv_message: Ok(String::new()),
            settings_database_message: Ok(String::new()),
            settings_more_message: Ok(String::new()),
            settings_server_choice: if saved_user
                .as_ref()
                .map_or(false, |user| user.server != OFFICIAL_SERVER)
            {
                Some(ServerChoices::Custom)
            } else {
                Some(ServerChoices::Official)
            },
            shortcuts: match db_retrieve_existing_shortcuts() {
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
            theme: FurTheme::Light,
            timer_is_running: false,
            timer_start_time: Local::now(),
            timer_text: "0:00:00".to_string(),
            task_to_add: None,
            task_to_edit: None,
        };

        furtherance.theme = get_system_theme(furtherance.fur_settings.theme);
        furtherance.timer_text = get_timer_text(&furtherance, 0);

        if autosave_exists() {
            restore_autosave();
            if furtherance.displayed_alert == None {
                furtherance.displayed_alert = Some(FurAlert::AutosaveRestored);
            }
        }

        // Ask user to import old Furtherance database on first run
        if furtherance.fur_settings.first_run {
            #[cfg(target_os = "macos")]
            {
                furtherance.displayed_alert = db_check_for_existing_mac_db();
            }

            let _ = furtherance.fur_settings.change_first_run(false);
            let _ = furtherance.fur_settings.change_notify_of_sync(false);
        } else if furtherance.fur_settings.notify_of_sync {
            furtherance.displayed_alert = Some(FurAlert::NotifyOfSync)
        }

        furtherance.task_history = get_task_history(furtherance.fur_settings.days_to_show);

        let user = furtherance.fur_user.clone();
        (
            furtherance,
            if user.is_some() {
                Task::perform(
                    async {
                        // Small delay to allow startup operations to complete
                        time::sleep(Duration::from_secs(1)).await;
                    },
                    |_| Message::SyncWithServer,
                )
            } else {
                Task::none()
            },
        )
    }

    pub fn title(&self) -> String {
        "Furtherance".to_owned()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone().to_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Live dark-light theme switching does not currently work on macOS
        #[cfg(not(target_os = "macos"))]
        let theme_watcher = if self.fur_settings.theme == FurDarkLight::Auto {
            Some(iced::time::every(time::Duration::from_secs(10)).map(|_| Message::ChangeTheme))
        } else {
            None
        };

        let show_reminder_notification = if self.fur_settings.notify_reminder {
            Some(
                iced::time::every(time::Duration::from_secs(
                    (self.fur_settings.notify_reminder_interval * 60) as u64,
                ))
                .map(|_| Message::ShowReminderNotification),
            )
        } else {
            None
        };

        let key_presssed = keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::Tab, _) => Some(Message::TabPressed {
                    shift: modifiers.shift(),
                }),
                _ => None,
            }
        });

        let timed_sync = if self.fur_user.is_some() {
            let sync_interval = 900; // 15 mins in secs
            Some(
                iced::time::every(Duration::from_secs(sync_interval))
                    .map(|_| Message::SyncWithServer),
            )
        } else {
            None
        };

        Subscription::batch([
            key_presssed,
            subscription::from_recipe(MidnightSubscription),
            show_reminder_notification.unwrap_or(Subscription::none()),
            timed_sync.unwrap_or(Subscription::none()),
            #[cfg(not(target_os = "macos"))]
            theme_watcher.unwrap_or(Subscription::none()),
        ])
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
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
                self.settings_database_message = Ok(String::new());
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
                            self.settings_database_message =
                                Ok(self.localization.get_message("backup-successful", None));
                        }
                        Err(_) => {
                            self.settings_database_message = Err(self
                                .localization
                                .get_message("backup-database-failed", None)
                                .into());
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
            Message::ChangeTheme => self.theme = get_system_theme(self.fur_settings.theme),
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
            Message::ClearLoginMessage => {
                self.login_message = Ok(String::new());
            }
            Message::CreateShortcutFromTaskGroup(task_group) => {
                let new_shortcut = FurShortcut::new(
                    task_group.name,
                    if task_group.tags.is_empty() {
                        String::new()
                    } else {
                        format!("#{}", task_group.tags)
                    },
                    task_group.project,
                    task_group.rate,
                    String::new(),
                    Srgb::random().to_hex(),
                );

                match db_shortcut_exists(&new_shortcut) {
                    Ok(exists) => {
                        if exists {
                            self.displayed_alert = Some(FurAlert::ShortcutExists);
                        } else {
                            if let Err(e) = db_insert_shortcut(&new_shortcut) {
                                eprintln!("Failed to write shortcut to database: {}", e);
                            }
                            match db_retrieve_existing_shortcuts() {
                                Ok(shortcuts) => self.shortcuts = shortcuts,
                                Err(e) => {
                                    eprintln!("Failed to retrieve shortcuts from database: {}", e)
                                }
                            };
                            self.current_view = FurView::Shortcuts;
                            return sync_after_change(&self.fur_user);
                        }
                    }
                    Err(e) => eprintln!("Failed to check if shortcut exists: {}", e),
                }
            }
            Message::DeleteEverything => match db_delete_everything() {
                Ok(_) => {
                    self.displayed_alert = None;
                    self.settings_more_message =
                        Ok(self.localization.get_message("deleted-everything", None));
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                    match db_retrieve_existing_shortcuts() {
                        Ok(shortcuts) => self.shortcuts = shortcuts,
                        Err(e) => {
                            eprintln!("Failed to retrieve shortcuts from database: {}", e)
                        }
                    };
                }
                Err(_) => {
                    self.settings_more_message = Err(self
                        .localization
                        .get_message("error-deleting-everything", None)
                        .into());
                }
            },
            Message::DateRangeSelected(new_range) => self.report.set_picked_date_ranged(new_range),
            Message::DeleteShortcut => {
                if let Some(uid) = &self.delete_shortcut_from_context {
                    if let Err(e) = db_delete_shortcut_by_id(uid) {
                        eprintln!("Failed to delete shortcut: {}", e);
                    }
                    self.delete_shortcut_from_context = None;
                    self.displayed_alert = None;
                    match db_retrieve_existing_shortcuts() {
                        Ok(shortcuts) => self.shortcuts = shortcuts,
                        Err(e) => eprintln!("Failed to retrieve shortcuts from database: {}", e),
                    };
                }
            }
            Message::DeleteShortcutFromContext(id) => {
                self.delete_shortcut_from_context = Some(id);
                let delete_confirmation = self.fur_settings.show_delete_confirmation;
                return Task::perform(
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
                    if let Err(e) = db_delete_tasks_by_ids(tasks_to_delete) {
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
                    if let Err(e) = db_delete_tasks_by_ids(&[task_to_edit.uid.clone()]) {
                        eprintln!("Failed to delete task: {}", e);
                    }
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                } else if let Some(group_to_edit) = &self.group_to_edit {
                    self.inspector_view = None;
                    if let Err(e) = db_delete_tasks_by_ids(&group_to_edit.all_task_ids()) {
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

                return Task::perform(
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
                                shortcut_to_add.input_error(
                                    self.localization.get_message("name-cannot-contain", None),
                                );
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
                                shortcut_to_add.input_error(
                                    self.localization
                                        .get_message("project-cannot-contain", None),
                                );
                            } else {
                                shortcut_to_add.project = new_value;
                                shortcut_to_add.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Tags => {
                            if new_value.contains('@') || new_value.contains('$') {
                                shortcut_to_add.input_error(
                                    self.localization.get_message("tags-cannot-contain", None),
                                );
                            } else if !new_value.is_empty() && new_value.chars().next() != Some('#')
                            {
                                shortcut_to_add.input_error(
                                    self.localization.get_message("tags-must-start", None),
                                );
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
                                shortcut_to_add.input_error(
                                    self.localization.get_message("no-symbol-in-rate", None),
                                );
                            } else if new_value_parsed.is_ok()
                                && has_max_two_decimals(&new_value)
                                && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                            {
                                shortcut_to_add.new_rate = new_value;
                                shortcut_to_add.input_error(String::new());
                            } else {
                                shortcut_to_add.input_error(
                                    self.localization.get_message("rate-invalid", None),
                                );
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
                                shortcut_to_edit.input_error(
                                    self.localization.get_message("name-cannot-contain", None),
                                )
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
                                shortcut_to_edit.input_error(
                                    self.localization
                                        .get_message("project-cannot-contain", None),
                                );
                            } else {
                                shortcut_to_edit.new_project = new_value;
                                shortcut_to_edit.input_error(String::new());
                            }
                        }
                        EditTaskProperty::Tags => {
                            if new_value.contains('@') || new_value.contains('$') {
                                shortcut_to_edit.input_error(
                                    self.localization.get_message("tags-cannot-contain", None),
                                );
                            } else if !new_value.is_empty() && new_value.chars().next() != Some('#')
                            {
                                shortcut_to_edit.input_error(
                                    self.localization.get_message("tags-must-start", None),
                                );
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
                                shortcut_to_edit.input_error(
                                    self.localization.get_message("no-symbol-in-rate", None),
                                );
                            } else if new_value_parsed.is_ok()
                                && has_max_two_decimals(&new_value)
                                && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                            {
                                shortcut_to_edit.new_rate = new_value;
                                shortcut_to_edit.input_error(String::new());
                            } else {
                                shortcut_to_edit.input_error(
                                    self.localization.get_message("rate-invalid", None),
                                );
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
            Message::EditTaskTextChanged(new_value, property) => match self.inspector_view {
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
                                    task_to_add.input_error(
                                        self.localization
                                            .get_message("project-cannot-contain", None),
                                    );
                                } else {
                                    task_to_add.project = new_value;
                                }
                            }
                            EditTaskProperty::Tags => {
                                if new_value.contains('@') || new_value.contains('$') {
                                    task_to_add.input_error(
                                        self.localization.get_message("tags-cannot-contain", None),
                                    );
                                } else if !new_value.is_empty()
                                    && new_value.chars().next() != Some('#')
                                {
                                    task_to_add.input_error(
                                        self.localization.get_message("tags-must-start", None),
                                    );
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
                                    task_to_add.input_error(
                                        self.localization.get_message("no-symbol-in-rate", None),
                                    );
                                } else if new_value_parsed.is_ok()
                                    && has_max_two_decimals(&new_value)
                                    && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                {
                                    task_to_add.new_rate = new_value;
                                    task_to_add.input_error(String::new());
                                } else {
                                    task_to_add.input_error(
                                        self.localization.get_message("rate-invalid", None),
                                    );
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
                                    task_to_edit.input_error(
                                        self.localization.get_message("name-cannot-contain", None),
                                    );
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
                                    task_to_edit.input_error(
                                        self.localization
                                            .get_message("project-cannot-contain", None),
                                    );
                                } else {
                                    task_to_edit.new_project = new_value;
                                }
                            }
                            EditTaskProperty::Tags => {
                                if new_value.contains('@') || new_value.contains('$') {
                                    task_to_edit.input_error(
                                        self.localization.get_message("tags-cannot-contain", None),
                                    );
                                } else if !new_value.is_empty()
                                    && new_value.chars().next() != Some('#')
                                {
                                    task_to_edit.input_error(
                                        self.localization.get_message("tags-must-start", None),
                                    );
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
                                    task_to_edit.input_error(
                                        self.localization.get_message("no-symbol-in-rate", None),
                                    );
                                } else if new_value_parsed.is_ok()
                                    && has_max_two_decimals(&new_value)
                                    && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                {
                                    task_to_edit.new_rate = new_value;
                                    task_to_edit.input_error(String::new());
                                } else {
                                    task_to_edit.input_error(
                                        self.localization.get_message("rate-invalid", None),
                                    );
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
                                    group_to_edit.input_error(
                                        self.localization.get_message("name-cannot-contain", None),
                                    );
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
                                    group_to_edit.input_error(
                                        self.localization
                                            .get_message("project-cannot-contain", None),
                                    );
                                } else {
                                    group_to_edit.new_project = new_value;
                                }
                            }
                            EditTaskProperty::Tags => {
                                if new_value.contains('@') || new_value.contains('$') {
                                    group_to_edit.input_error(
                                        self.localization.get_message("tags-cannot-contain", None),
                                    );
                                } else if !new_value.is_empty()
                                    && new_value.chars().next() != Some('#')
                                {
                                    group_to_edit.input_error(
                                        self.localization.get_message("tags-must-start", None),
                                    );
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
                                    group_to_edit.input_error(
                                        self.localization.get_message("no-symbol-in-rate", None),
                                    );
                                } else if new_value_parsed.is_ok()
                                    && has_max_two_decimals(&new_value)
                                    && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                {
                                    group_to_edit.new_rate = new_value;
                                    group_to_edit.input_error(String::new());
                                } else {
                                    group_to_edit.input_error(
                                        self.localization.get_message("rate-invalid", None),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Message::EnterPressedInTaskInput => {
                if !self.task_input.is_empty() {
                    if !self.timer_is_running {
                        return Task::perform(async { Message::StartStopPressed }, |msg| msg);
                    }
                }
            }
            Message::EnterPressedInSyncFields => {
                if !self.fur_user_fields.server.is_empty()
                    && !self.fur_user_fields.email.is_empty()
                    && !self.fur_user_fields.encryption_key.is_empty()
                {
                    return Task::perform(async { Message::UserLoginPressed }, |msg| msg);
                }
            }
            Message::ExportCsvPressed => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_message = Ok(String::new());
                let file_name = format!("furtherance-{}.csv", Local::now().format("%Y-%m-%d"));
                let selected_file = FileDialog::new()
                    .set_title(self.localization.get_message("save-csv-title", None))
                    .add_filter("CSV", &["csv"])
                    .set_can_create_directories(true)
                    .set_file_name(file_name)
                    .save_file();

                if let Some(path) = selected_file {
                    match write_furtasks_to_csv(path, &self.localization) {
                        Ok(_) => {
                            self.settings_csv_message =
                                Ok(self.localization.get_message("csv-file-saved", None))
                        }
                        Err(e) => {
                            eprintln!("Error writing data to CSV: {}", e);
                            self.settings_csv_message = Err(self
                                .localization
                                .get_message("error-writing-csv", None)
                                .into());
                        }
                    }
                }
            }
            Message::FontLoaded(_) => {}
            Message::IdleDiscard => {
                stop_timer(self, self.idle.start_time);
                self.displayed_alert = None;
                return sync_after_change(&self.fur_user);
            }
            Message::IdleReset => {
                self.idle = FurIdle::new();
                self.displayed_alert = None;
            }
            Message::ImportCsvPressed => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_message = Ok(String::new());
                let selected_file = FileDialog::new()
                    .set_title(self.localization.get_message("open-csv-title", None))
                    .add_filter("CSV", &["csv"])
                    .set_can_create_directories(false)
                    .pick_file();
                if let Some(path) = selected_file {
                    if let Ok(mut file) = File::open(path) {
                        match verify_csv(&file, &self.localization) {
                            Ok(_) => {
                                import_csv_to_database(&mut file, &self.localization);
                                self.settings_csv_message =
                                    Ok(self.localization.get_message("csv-imported", None).into());

                                // Always do a full sync after import
                                if let Err(e) = self.fur_settings.change_needs_full_sync(&true) {
                                    eprintln!("Error changing needs_full_sync: {}", e);
                                };

                                self.task_history =
                                    get_task_history(self.fur_settings.days_to_show);
                            }
                            Err(e) => {
                                eprintln!("Invalid CSV file: {}", e);
                                self.settings_csv_message = Err(self
                                    .localization
                                    .get_message("invalid-csv-file", None)
                                    .into());
                            }
                        }
                    }
                }
            }
            Message::ImportOldMacDatabase => {
                match db_import_old_mac_db() {
                    Ok(_) => {
                        // Always do a full sync after import
                        if let Err(e) = self.fur_settings.change_needs_full_sync(&true) {
                            eprintln!("Error changing needs_full_sync: {}", e);
                        };

                        self.task_history = get_task_history(self.fur_settings.days_to_show)
                    }
                    Err(e) => eprintln!("Error importing existing Core Data database: {e}"),
                }
                self.displayed_alert = None;
            }
            Message::LearnAboutSync => {
                if let Err(e) = webbrowser::open("https://furtherance.app/sync") {
                    eprintln!("Failed to open URL in browser: {}", e);
                }
                self.displayed_alert = None;
                if let Err(e) = self.fur_settings.change_notify_of_sync(false) {
                    eprintln!("Error changing notify_of_sync: {}", e);
                };
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
            Message::NotifyOfSyncClose => {
                if let Err(e) = self.fur_settings.change_notify_of_sync(false) {
                    eprintln!("Error changing notify_of_sync: {}", e);
                };
                return Task::perform(async { Message::AlertClose }, |msg| msg);
            }
            Message::OpenUrl(url) => {
                if let Err(e) = webbrowser::open(&url) {
                    eprintln!("Failed to open URL in browser: {}", e);
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
                return Task::perform(get_timer_duration(), |_| Message::StopwatchTick);
            }
            Message::PomodoroSnooze => {
                self.pomodoro.snoozed = true;
                self.pomodoro.snoozed_at = Local::now();
                // Timer is still running but we want to first show the snooze time total
                self.timer_text = get_stopped_timer_text(self);
                self.displayed_alert = None;
                return Task::perform(get_timer_duration(), |_| Message::StopwatchTick);
            }
            Message::PomodoroStartBreak => {
                let original_task_input = self.task_input.clone();
                self.pomodoro.on_break = true;
                self.pomodoro.snoozed = false;
                stop_timer(self, Local::now());
                self.task_input = original_task_input;
                self.displayed_alert = None;
                start_timer(self);
                return Task::batch([
                    Task::perform(get_timer_duration(), |_| Message::StopwatchTick),
                    sync_after_change(&self.fur_user),
                ]);
            }
            Message::PomodoroStop => {
                self.pomodoro.snoozed = false;
                stop_timer(self, Local::now());
                self.displayed_alert = None;
                self.pomodoro.sessions = 0;
                return sync_after_change(&self.fur_user);
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
                return Task::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::ReportTabSelected(new_tab) => self.report.active_tab = new_tab,
            Message::SaveGroupEdit => {
                if let Some(group_to_edit) = &self.group_to_edit {
                    let _ = db_update_group_of_tasks(group_to_edit);
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    self.task_history = get_task_history(self.fur_settings.days_to_show);
                    return sync_after_change(&self.fur_user);
                }
            }
            Message::SaveShortcut => {
                if let Some(shortcut_to_add) = &self.shortcut_to_add {
                    let new_shortcut = FurShortcut::new(
                        shortcut_to_add.name.trim().to_string(),
                        shortcut_to_add.tags.trim().to_string(),
                        shortcut_to_add.project.trim().to_string(),
                        shortcut_to_add
                            .new_rate
                            .trim()
                            .parse::<f32>()
                            .unwrap_or(0.0),
                        String::new(),
                        shortcut_to_add.color.to_hex(),
                    );
                    match db_shortcut_exists(&new_shortcut) {
                        Ok(exists) => {
                            if exists {
                                self.displayed_alert = Some(FurAlert::ShortcutExists);
                            } else {
                                match db_insert_shortcut(&new_shortcut) {
                                    Ok(_) => {
                                        self.inspector_view = None;
                                        self.shortcut_to_add = None;
                                        match db_retrieve_existing_shortcuts() {
                                            Ok(shortcuts) => self.shortcuts = shortcuts,
                                            Err(e) => eprintln!(
                                                "Failed to retrieve shortcuts from database: {}",
                                                e
                                            ),
                                        };
                                        return sync_after_change(&self.fur_user);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to write shortcut to database: {}", e)
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("Failed to check if shortcut exists: {}", e),
                    }
                } else if let Some(shortcut_to_edit) = &self.shortcut_to_edit {
                    match db_update_shortcut(&FurShortcut {
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
                        uid: shortcut_to_edit.uid.clone(),
                        is_deleted: false,
                        last_updated: chrono::Utc::now().timestamp(),
                    }) {
                        Ok(_) => {
                            self.inspector_view = None;
                            self.shortcut_to_edit = None;
                            match db_retrieve_existing_shortcuts() {
                                Ok(shortcuts) => self.shortcuts = shortcuts,
                                Err(e) => {
                                    eprintln!("Failed to retrieve shortcuts from database: {}", e)
                                }
                            };
                            return sync_after_change(&self.fur_user);
                        }
                        Err(e) => eprintln!("Failed to update shortcut in database: {}", e),
                    }
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
                    match db_update_task(&FurTask {
                        name: task_to_edit.new_name.trim().to_string(),
                        start_time: task_to_edit.new_start_time,
                        stop_time: task_to_edit.new_stop_time,
                        tags: tags_without_first_pound,
                        project: task_to_edit.new_project.trim().to_string(),
                        rate: task_to_edit.new_rate.trim().parse::<f32>().unwrap_or(0.0),
                        currency: String::new(),
                        uid: task_to_edit.uid.clone(),
                        is_deleted: false,
                        last_updated: chrono::Utc::now().timestamp(),
                    }) {
                        Ok(_) => {
                            self.inspector_view = None;
                            self.task_to_edit = None;
                            self.group_to_edit = None;
                            self.task_history = get_task_history(self.fur_settings.days_to_show);
                            return sync_after_change(&self.fur_user);
                        }
                        Err(e) => eprintln!("Failed to update task in database: {}", e),
                    }
                } else if let Some(task_to_add) = &self.task_to_add {
                    let tags_without_first_pound = task_to_add
                        .tags
                        .trim()
                        .strip_prefix('#')
                        .unwrap_or(&task_to_add.tags)
                        .trim()
                        .to_string();
                    match db_insert_task(&FurTask::new(
                        task_to_add.name.trim().to_string(),
                        task_to_add.start_time,
                        task_to_add.stop_time,
                        tags_without_first_pound,
                        task_to_add.project.trim().to_string(),
                        task_to_add.new_rate.trim().parse::<f32>().unwrap_or(0.0),
                        String::new(),
                    )) {
                        Ok(_) => {
                            self.inspector_view = None;
                            self.task_to_add = None;
                            self.group_to_edit = None;
                            self.task_history = get_task_history(self.fur_settings.days_to_show);
                            return sync_after_change(&self.fur_user);
                        }
                        Err(e) => eprintln!("Error adding task: {}", e),
                    }
                }
            }
            Message::SettingsChangeDatabaseLocationPressed(new_or_open) => {
                self.settings_csv_message = Ok(String::new());
                self.settings_database_message = Ok(String::new());
                let path = Path::new(&self.fur_settings.database_url);
                let starting_dialog = FileDialog::new()
                    .set_directory(&path)
                    .add_filter(
                        self.localization.get_message("sqlite-files", None),
                        ALLOWED_DB_EXTENSIONS,
                    )
                    .set_can_create_directories(true);

                let selected_file = match new_or_open {
                    ChangeDB::New => starting_dialog
                        .set_file_name("furtherance.db")
                        .set_title(self.localization.get_message("new-database-title", None))
                        .save_file(),
                    ChangeDB::Open => starting_dialog
                        .set_title(self.localization.get_message("open-database-title", None))
                        .pick_file(),
                };

                let mut is_old_db = false;

                if let Some(file) = selected_file {
                    self.settings_database_message = Ok(String::new());

                    if file.exists() {
                        match db_is_valid_v3(file.as_path()) {
                            Err(e) => {
                                eprintln!("Invalid database: {}", e);
                                self.settings_database_message = Err(self
                                    .localization
                                    .get_message("invalid-database", None)
                                    .into());
                            }
                            Ok(is_valid_v3) => {
                                if !is_valid_v3 {
                                    match db_is_valid_v1(file.as_path()) {
                                        Ok(is_valid_v2) => {
                                            if is_valid_v2 {
                                                is_old_db = true
                                            } else {
                                                self.settings_database_message = Err(self
                                                    .localization
                                                    .get_message("invalid-database", None)
                                                    .into());
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Invalid v1 database: {}", e);
                                            self.settings_database_message = Err(self
                                                .localization
                                                .get_message("invalid-database", None)
                                                .into());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if self.settings_database_message.is_ok() {
                        // Valid file or not yet a file
                        if let Some(file_str) = file.to_str() {
                            if let Ok(_) = self.fur_settings.change_db_url(file_str) {
                                match db_init() {
                                    Ok(_) => {
                                        if is_old_db {
                                            if let Err(e) = db_upgrade_old() {
                                                eprintln!("Error upgrading legacy database: {}", e);
                                                self.settings_database_message = Err(self
                                                    .localization
                                                    .get_message("error-upgrading-database", None)
                                                    .into());
                                                return Task::none();
                                            }
                                        }
                                        self.task_history =
                                            get_task_history(self.fur_settings.days_to_show);
                                        self.settings_database_message = Ok(match new_or_open {
                                            ChangeDB::Open => self
                                                .localization
                                                .get_message("database-loaded", None),
                                            ChangeDB::New => self
                                                .localization
                                                .get_message("database-created", None)
                                                .to_string(),
                                        });
                                    }
                                    Err(e) => {
                                        eprintln!("Error accessing new database: {}", e);
                                        self.settings_database_message = Err(self
                                            .localization
                                            .get_message("error-accessing-database", None)
                                            .into());
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
            Message::SettingsPomodoroNotificationAlarmSoundToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_pomodoro_notification_alarm_sound(&new_value) {
                    eprintln!("Failed to change pomodoro_notification_alarm_sound in settings: {}", e);
                }
            }
            Message::SettingsReminderIntervalChanged(new_value) => {
                if let Err(e) = self
                    .fur_settings
                    .change_notify_reminder_interval(&new_value)
                {
                    eprintln!(
                        "Failed to change notify_reminder_interval in settings: {}",
                        e
                    );
                }
            }
            Message::SettingsRemindersToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_notify_reminder(&new_value) {
                    eprintln!("Failed to change notify_reminder in settings: {}", e);
                }
            }
            Message::ShowReminderNotification => {
                if !self.timer_is_running {
                    show_notification(NotificationType::Reminder, &self.localization, self.fur_settings.pomodoro_notification_alarm_sound);
                }
            }
            Message::SettingsServerChoiceSelected(new_value) => {
                self.settings_server_choice = Some(new_value);
                if new_value == ServerChoices::Official {
                    self.fur_user_fields.server = OFFICIAL_SERVER.to_string();
                } else {
                    if let Some(fur_user) = &self.fur_user {
                        self.fur_user_fields.server = fur_user.server.clone();
                    } else {
                        self.fur_user_fields.server = String::new();
                    }
                }
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
            Message::SettingsThemeSelected(selected_theme) => {
                if let Err(e) = self.fur_settings.change_theme(&selected_theme) {
                    eprintln!("Failed to change theme in settings: {}", e);
                }
                self.theme = get_system_theme(self.fur_settings.theme)
            }
            Message::ShortcutPressed(shortcut_task_input) => {
                self.task_input = shortcut_task_input;
                self.inspector_view = None;
                self.shortcut_to_add = None;
                self.shortcut_to_edit = None;
                self.current_view = FurView::Timer;
                return Task::perform(async { Message::StartStopPressed }, |msg| msg);
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
                    // Do not move declarations to after if else
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
                        return sync_after_change(&self.fur_user);
                    }
                    return Task::none();
                } else {
                    start_timer(self);
                    return Task::perform(get_timer_duration(), |_| Message::StopwatchTick);
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
                                show_notification(NotificationType::BreakOver, &self.localization, self.fur_settings.pomodoro_notification_alarm_sound);
                                self.displayed_alert = Some(FurAlert::PomodoroBreakOver);
                            } else {
                                show_notification(
                                    NotificationType::PomodoroOver,
                                    &self.localization,
                                    self.fur_settings.pomodoro_notification_alarm_sound,
                                );
                                self.displayed_alert = Some(FurAlert::PomodoroOver);
                            }
                        }
                        return Task::none();
                    }

                    if self.fur_settings.notify_on_idle
                        && self.displayed_alert != Some(FurAlert::PomodoroOver)
                    {
                        let idle_time = idle::get_idle_time() as i64;
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
                            show_notification(NotificationType::Idle, &self.localization, self.fur_settings.pomodoro_notification_alarm_sound);
                            self.displayed_alert = Some(FurAlert::Idle);
                        }
                    }

                    // Write autosave every minute
                    if seconds_elapsed > 1 && seconds_elapsed % 60 == 0 {
                        if let Err(e) = write_autosave(&self.task_input, self.timer_start_time) {
                            eprintln!("Error writing autosave: {e}");
                        }
                    }

                    return Task::perform(get_timer_duration(), |_| Message::StopwatchTick);
                } else {
                    return Task::none();
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
            Message::SyncWithServer => {
                let last_sync = self.fur_settings.last_sync;

                let user = match self.fur_user.clone() {
                    Some(user) => user,
                    None => {
                        eprintln!("Please log in first");
                        return Task::none();
                    }
                };

                self.login_message = Ok(self.localization.get_message("syncing", None));

                let encryption_key =
                    match decrypt_encryption_key(&user.encrypted_key, &user.key_nonce) {
                        Ok(key) => key,
                        Err(e) => {
                            eprintln!("Failed to decrypt encryption key (SyncWithServer): {:?}", e);
                            return set_negative_temp_notice(
                                &mut self.login_message,
                                self.localization.get_message("error-decrypting-key", None),
                            );
                        }
                    };

                let needs_full_sync = self.fur_settings.needs_full_sync;

                return Task::perform(
                    async move {
                        let new_tasks: Vec<FurTask>;
                        let new_shortcuts: Vec<FurShortcut>;

                        if needs_full_sync {
                            new_tasks =
                                db_retrieve_all_tasks(SortBy::StartTime, SortOrder::Ascending)
                                    .unwrap_or_default();
                            new_shortcuts = db_retrieve_all_shortcuts().unwrap_or_default();
                        } else {
                            new_tasks =
                                db_retrieve_tasks_since_timestamp(last_sync).unwrap_or_default();
                            new_shortcuts = db_retrieve_shortcuts_since_timestamp(last_sync)
                                .unwrap_or_default();
                        }

                        let encrypted_tasks: Vec<EncryptedTask> = new_tasks
                            .into_iter()
                            .filter_map(|task| match encryption::encrypt(&task, &encryption_key) {
                                Ok((encrypted_data, nonce)) => Some(EncryptedTask {
                                    encrypted_data,
                                    nonce,
                                    uid: task.uid,
                                    last_updated: task.last_updated,
                                }),
                                Err(e) => {
                                    eprintln!("Failed to encrypt task: {:?}", e);
                                    None
                                }
                            })
                            .collect();

                        let encrypted_shortcuts: Vec<EncryptedShortcut> = new_shortcuts
                            .into_iter()
                            .filter_map(|shortcut| {
                                match encryption::encrypt(&shortcut, &encryption_key) {
                                    Ok((encrypted_data, nonce)) => Some(EncryptedShortcut {
                                        encrypted_data,
                                        nonce,
                                        uid: shortcut.uid,
                                        last_updated: shortcut.last_updated,
                                    }),
                                    Err(e) => {
                                        eprintln!("Failed to encrypt shortcut: {:?}", e);
                                        None
                                    }
                                }
                            })
                            .collect();

                        let sync_count = encrypted_tasks.len() + encrypted_shortcuts.len();

                        let sync_result = sync_with_server(
                            &user,
                            last_sync,
                            encrypted_tasks,
                            encrypted_shortcuts,
                        )
                        .await;

                        (sync_result, sync_count)
                    },
                    Message::SyncComplete,
                );
            }
            Message::SyncComplete(sync_result) => {
                match sync_result {
                    (Ok(response), mut sync_count) => {
                        let user = match self.fur_user.clone() {
                            Some(user) => user,
                            None => {
                                eprintln!("Please log in first");
                                return set_negative_temp_notice(
                                    &mut self.login_message,
                                    self.localization.get_message("log-in-first", None),
                                );
                            }
                        };

                        let encryption_key =
                            match decrypt_encryption_key(&user.encrypted_key, &user.key_nonce) {
                                Ok(key) => key,
                                Err(e) => {
                                    eprintln!(
                                        "Failed to decrypt encryption key (SyncComplete): {:?}",
                                        e
                                    );
                                    return set_negative_temp_notice(
                                        &mut self.login_message,
                                        self.localization.get_message("error-decrypting-key", None),
                                    );
                                }
                            };

                        // Decrypt and process server tasks
                        for encrypted_task in response.tasks {
                            match encryption::decrypt::<FurTask>(
                                &encrypted_task.encrypted_data,
                                &encrypted_task.nonce,
                                &encryption_key,
                            ) {
                                Ok(server_task) => {
                                    match db_retrieve_task_by_id(&server_task.uid) {
                                        Ok(Some(client_task)) => {
                                            // Task exists - update it if it changed
                                            if server_task.last_updated > client_task.last_updated {
                                                if let Err(e) = db_update_task(&server_task) {
                                                    eprintln!(
                                                        "Error updating task from server: {}",
                                                        e
                                                    );
                                                } else {
                                                    sync_count += 1;
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            // Task does not exist - insert it
                                            if let Err(e) = db_insert_task(&server_task) {
                                                eprintln!(
                                                    "Error writing new task from server: {}",
                                                    e
                                                );
                                            } else {
                                                sync_count += 1;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error checking for existing task from server: {}",
                                                e
                                            )
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to decrypt task: {:?}", e),
                            }
                        }

                        // Decrypt and process server shortcuts
                        for encrypted_shortcut in response.shortcuts {
                            match encryption::decrypt::<FurShortcut>(
                                &encrypted_shortcut.encrypted_data,
                                &encrypted_shortcut.nonce,
                                &encryption_key,
                            ) {
                                Ok(server_shortcut) => {
                                    match db_retrieve_shortcut_by_id(&server_shortcut.uid) {
                                        Ok(Some(client_shortcut)) => {
                                            // Shortcut exists - update it if it changed
                                            if server_shortcut.last_updated
                                                > client_shortcut.last_updated
                                            {
                                                if let Err(e) = db_update_shortcut(&server_shortcut)
                                                {
                                                    eprintln!(
                                                        "Error updating shortcut from server: {}",
                                                        e
                                                    );
                                                } else {
                                                    sync_count += 1;
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            // Shortcut does not exist - insert it
                                            if let Err(e) = db_insert_shortcut(&server_shortcut) {
                                                eprintln!(
                                                    "Error writing new shortcut from server: {}",
                                                    e
                                                );
                                            } else {
                                                sync_count += 1;
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error checking for existing shortcut from server: {}",
                                                e
                                            )
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to decrypt shortcut: {:?}", e),
                            }
                        }

                        // Update last sync timestamp
                        if let Err(e) = self
                            .fur_settings
                            .change_last_sync(&response.server_timestamp)
                        {
                            eprintln!("Failed to change last_sync in settings: {}", e);
                        }

                        // If the database_id changed, send all tasks, or if the server has orphaned tasks, re-sync those
                        if !response.orphaned_tasks.is_empty()
                            || !response.orphaned_shortcuts.is_empty()
                        {
                            let last_sync = self.fur_settings.last_sync;

                            let orphaned_tasks = if !response.orphaned_tasks.is_empty() {
                                db_retrieve_orphaned_tasks(response.orphaned_tasks)
                                    .unwrap_or_default()
                            } else {
                                Vec::new()
                            };

                            let orphaned_shortcuts = if !response.orphaned_shortcuts.is_empty() {
                                db_retrieve_orphaned_shortcuts(response.orphaned_shortcuts)
                                    .unwrap_or_default()
                            } else {
                                Vec::new()
                            };

                            if !orphaned_tasks.is_empty() || !orphaned_shortcuts.is_empty() {
                                return Task::perform(
                                    async move {
                                        let encrypted_tasks: Vec<EncryptedTask> = orphaned_tasks
                                            .into_iter()
                                            .filter_map(|task| {
                                                match encryption::encrypt(&task, &encryption_key) {
                                                    Ok((encrypted_data, nonce)) => {
                                                        Some(EncryptedTask {
                                                            encrypted_data,
                                                            nonce,
                                                            uid: task.uid,
                                                            last_updated: task.last_updated,
                                                        })
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to encrypt task: {:?}",
                                                            e
                                                        );
                                                        None
                                                    }
                                                }
                                            })
                                            .collect();

                                        let encrypted_shortcuts: Vec<EncryptedShortcut> =
                                            orphaned_shortcuts
                                                .into_iter()
                                                .filter_map(|shortcut| {
                                                    match encryption::encrypt(
                                                        &shortcut,
                                                        &encryption_key,
                                                    ) {
                                                        Ok((encrypted_data, nonce)) => {
                                                            Some(EncryptedShortcut {
                                                                encrypted_data,
                                                                nonce,
                                                                uid: shortcut.uid,
                                                                last_updated: shortcut.last_updated,
                                                            })
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to encrypt shortcut: {:?}",
                                                                e
                                                            );
                                                            None
                                                        }
                                                    }
                                                })
                                                .collect();

                                        sync_count +=
                                            encrypted_tasks.len() + encrypted_shortcuts.len();

                                        let sync_result = sync_with_server(
                                            &user,
                                            last_sync,
                                            encrypted_tasks,
                                            encrypted_shortcuts,
                                        )
                                        .await;

                                        (sync_result, sync_count)
                                    },
                                    Message::SyncComplete,
                                );
                            }
                        }

                        self.fur_settings.needs_full_sync = false;

                        self.task_history = get_task_history(self.fur_settings.days_to_show);

                        return set_positive_temp_notice(
                            &mut self.login_message,
                            self.localization.get_message(
                                "sync-successful",
                                Some(&HashMap::from([("count", FluentValue::from(sync_count))])),
                            ),
                        );
                    }
                    (Err(ApiError::TokenRefresh(msg)), _) if msg == "Failed to refresh token" => {
                        eprintln!("Sync error. Credentials have changed. Log in again.");
                        if let Some(user) = self.fur_user.clone() {
                            return Task::perform(
                                async move { logout::server_logout(&user).await },
                                |_| Message::UserAutoLogoutComplete,
                            );
                        }
                    }
                    (Err(ApiError::InactiveSubscription(msg)), _) => {
                        eprintln!("Sync error: {}", msg);
                        return set_negative_temp_notice(
                            &mut self.login_message,
                            self.localization.get_message("subscription-inactive", None),
                        );
                    }
                    (Err(e), _) => {
                        eprintln!("Sync error: {:?}", e);
                        return set_negative_temp_notice(
                            &mut self.login_message,
                            self.localization.get_message("sync-failed", None),
                        );
                    }
                }
            }
            Message::TabPressed { shift } => {
                if shift {
                    return widget::focus_previous();
                } else {
                    return widget::focus_next();
                }
            }
            Message::TaskInputChanged(new_value) => {
                // Handle all possible task input checks here rather than on start/stop press
                // If timer is running, task can never be empty
                if self.timer_is_running {
                    if new_value.trim().is_empty() {
                        return Task::none();
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
            Message::UserEmailChanged(new_email) => {
                self.fur_user_fields.email = new_email;
            }
            Message::UserLoginPressed => {
                self.fur_user_fields.server = self
                    .fur_user_fields
                    .server
                    .clone()
                    .trim_end_matches('/')
                    .to_string();
                let email = self.fur_user_fields.email.clone();
                let encryption_key = self.fur_user_fields.encryption_key.clone();
                let server = self.fur_user_fields.server.clone();
                self.login_message = Ok(self.localization.get_message("logging-in", None));

                return Task::perform(
                    login(email, encryption_key, server),
                    Message::UserLoginComplete,
                );
            }
            Message::UserLoginComplete(response_result) => match response_result {
                Ok(response) => {
                    // Encrypt encryption key with device-specific key
                    let (encrypted_key, key_nonce) =
                        match encrypt_encryption_key(&self.fur_user_fields.encryption_key) {
                            Ok(result) => result,
                            Err(e) => {
                                eprintln!("Error encrypting key: {:?}", e);
                                reset_fur_user(&mut self.fur_user);
                                return set_negative_temp_notice(
                                    &mut self.login_message,
                                    self.localization
                                        .get_message("error-storing-credentials", None),
                                );
                            }
                        };

                    // Store credentials
                    if let Err(e) = db_store_credentials(
                        &self.fur_user_fields.email,
                        &encrypted_key,
                        &key_nonce,
                        &response.access_token,
                        &response.refresh_token,
                        &self.fur_user_fields.server,
                    ) {
                        eprintln!("Error storing user credentials: {}", e);
                        reset_fur_user(&mut self.fur_user);
                        return set_negative_temp_notice(
                            &mut self.login_message,
                            self.localization
                                .get_message("error-storing-credentials", None),
                        );
                    }

                    // Always do a full sync after login
                    if let Err(e) = self.fur_settings.change_needs_full_sync(&true) {
                        eprintln!("Error changing needs_full_sync: {}", e);
                    };

                    let key_length = self.fur_user_fields.encryption_key.len();

                    // Load new user credentials from database
                    match db_retrieve_credentials() {
                        Ok(optional_user) => self.fur_user = optional_user,
                        Err(e) => {
                            eprintln!("Error retrieving user credentials from database: {}", e);
                            reset_fur_user(&mut self.fur_user);
                            return set_negative_temp_notice(
                                &mut self.login_message,
                                self.localization
                                    .get_message("error-storing-credentials", None),
                            );
                        }
                    };

                    if let Some(fur_user) = self.fur_user.clone() {
                        self.fur_user_fields.email = fur_user.email;
                        self.fur_user_fields.encryption_key = "x".repeat(key_length);
                        self.fur_user_fields.server = fur_user.server;
                        return set_positive_temp_notice(
                            &mut self.login_message,
                            self.localization.get_message("login-successful", None),
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error logging in: {:?}", e);
                    reset_fur_user(&mut self.fur_user);
                    match e {
                        ApiError::Network(e) if e.to_string() == "builder error" => {
                            return set_negative_temp_notice(
                                &mut self.login_message,
                                self.localization
                                    .get_message("server-must-contain-protocol", None),
                            );
                        }
                        _ => {
                            return set_negative_temp_notice(
                                &mut self.login_message,
                                self.localization.get_message("login-failed", None),
                            );
                        }
                    }
                }
            },
            Message::UserLogoutPressed => {
                // Send logout to server
                if let Some(user) = self.fur_user.clone() {
                    return Task::perform(
                        async move { logout::server_logout(&user).await },
                        |_| Message::UserLogoutComplete,
                    );
                }
            }
            Message::UserLogoutComplete => {
                reset_fur_user(&mut self.fur_user);
                self.fur_user_fields = FurUserFields::default();
                self.settings_server_choice = Some(ServerChoices::Official);
                return set_positive_temp_notice(
                    &mut self.login_message,
                    self.localization.get_message("logged-out", None),
                );
            }
            Message::UserAutoLogoutComplete => {
                reset_fur_user(&mut self.fur_user);
                self.fur_user_fields = FurUserFields::default();
                self.settings_server_choice = Some(ServerChoices::Official);
                return set_negative_temp_notice(
                    &mut self.login_message,
                    self.localization.get_message("reauthenticate-error", None),
                );
            }
            Message::UserEncryptionKeyChanged(new_key) => {
                self.fur_user_fields.encryption_key = new_key;
            }
            Message::UserServerChanged(new_server) => {
                self.fur_user_fields.server = new_server;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        // MARK: SIDEBAR
        let sidebar = Container::new(
            column![
                nav_button(
                    self.localization.get_message("shortcuts", None),
                    FurView::Shortcuts,
                    self.current_view == FurView::Shortcuts
                ),
                nav_button(
                    self.localization.get_message("timer", None),
                    FurView::Timer,
                    self.current_view == FurView::Timer
                ),
                nav_button(
                    self.localization.get_message("history", None),
                    FurView::History,
                    self.current_view == FurView::History
                ),
                nav_button(
                    self.localization.get_message("report", None),
                    FurView::Report,
                    self.current_view == FurView::Report
                ),
                vertical_space().height(Length::Fill),
                if self.timer_is_running && self.current_view != FurView::Timer {
                    text(convert_timer_text_to_vertical_hms(
                        &self.timer_text,
                        &self.localization,
                    ))
                    .size(50)
                    .style(|theme| {
                        if self.pomodoro.on_break {
                            style::red_text(theme)
                        } else {
                            text::Style::default()
                        }
                    })
                } else {
                    text("")
                },
                nav_button(
                    self.localization.get_message("settings", None),
                    FurView::Settings,
                    self.current_view == FurView::Settings
                ),
            ]
            .spacing(12)
            .align_x(Alignment::Start),
        )
        .width(175)
        .padding(10)
        .clip(true)
        .style(style::gray_background);

        // MARK: Shortcuts
        let mut shortcuts_row = Row::new().spacing(20.0);
        for shortcut in &self.shortcuts {
            shortcuts_row = shortcuts_row.push(shortcut_button(
                shortcut,
                self.timer_is_running,
                &self.localization,
            ));
        }
        let shortcuts_view = column![
            row![
                horizontal_space(),
                button(text(icon_to_char(Bootstrap::PlusLg)).font(BOOTSTRAP_FONT))
                    .on_press(Message::AddNewShortcutPressed)
                    .style(button::text),
            ]
            .padding([10, 20]),
            Scrollable::new(column![shortcuts_row.width(Length::Fill).wrap()].padding(20))
        ];

        // MARK: TIMER
        let timer_view = column![
            row![
                button(text(icon_to_char(Bootstrap::ArrowRepeat)).font(BOOTSTRAP_FONT))
                    .on_press_maybe(get_last_task_input(&self))
                    .style(button::text),
                horizontal_space().width(Length::Fill),
                text(self.localization.get_message(
                    "recorded-today",
                    Some(&HashMap::from([(
                        "time",
                        FluentValue::from(get_todays_total_time(&self))
                    )]))
                )),
            ],
            vertical_space().height(Length::Fill),
            text(&self.timer_text).size(80).style(|theme| {
                if self.pomodoro.on_break {
                    style::red_text(theme)
                } else {
                    text::Style::default()
                }
            }),
            column![
                row![
                    text_input(
                        &self
                            .localization
                            .get_message("task-input-placeholder", None),
                        &self.task_input
                    )
                    .on_input(Message::TaskInputChanged)
                    .on_submit(Message::EnterPressedInTaskInput)
                    .size(20),
                    button(row![
                        horizontal_space().width(Length::Fixed(5.0)),
                        if self.timer_is_running {
                            text(icon_to_char(Bootstrap::StopFill))
                                .font(BOOTSTRAP_FONT)
                                .size(20)
                        } else {
                            text(icon_to_char(Bootstrap::PlayFill))
                                .font(BOOTSTRAP_FONT)
                                .size(20)
                        },
                        horizontal_space().width(Length::Fixed(5.0)),
                    ])
                    .on_press_maybe(if self.task_input.trim().is_empty() {
                        None
                    } else {
                        Some(Message::StartStopPressed)
                    })
                    .style(style::primary_button_style),
                ]
                .spacing(10),
                if self.timer_is_running {
                    row![TimePicker::new(
                        self.show_timer_start_picker,
                        self.displayed_task_start_time,
                        Button::new(text(self.localization.get_message(
                            "started-at",
                            Some(&HashMap::from([(
                                "time",
                                FluentValue::from(
                                    self.timer_start_time.format("%H:%M").to_string()
                                )
                            )]))
                        )))
                        .on_press(Message::ChooseCurrentTaskStartTime)
                        .style(style::primary_button_style),
                        Message::CancelCurrentTaskStartTime,
                        Message::SubmitCurrentTaskStartTime,
                    )
                    .use_24h(),]
                    .align_y(Alignment::Center)
                    .spacing(10)
                } else {
                    row![button("").style(button::text)] // Button to match height
                },
            ]
            .align_x(Alignment::Center)
            .spacing(15),
            vertical_space().height(Length::Fill),
        ]
        .align_x(Alignment::Center)
        .padding(20);

        // MARK: HISTORY
        let mut all_history_rows: Column<'_, Message, Theme, Renderer> =
            Column::new().spacing(8).padding(20);
        if self.inspector_view.is_none() {
            all_history_rows = all_history_rows.push(row![
                horizontal_space(),
                button(text(icon_to_char(Bootstrap::PlusLg)).font(BOOTSTRAP_FONT))
                    .on_press(Message::AddNewTaskPressed)
                    .style(button::text),
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
                &self.localization,
            ));
            for task_group in task_groups {
                all_history_rows = all_history_rows.push(history_group_row(
                    task_group,
                    self.timer_is_running,
                    &self.fur_settings,
                    &self.localization,
                ))
            }
        }
        let history_view = column![Scrollable::new(all_history_rows)
            .width(Length::FillPortion(3)) // TODO: Adjust?
            .height(Length::Fill)];

        // MARK: REPORT
        let mut charts_column = Column::new().align_x(Alignment::Center);

        let mut timer_earnings_boxes_widgets: Vec<Element<'_, Message, Theme, Renderer>> =
            Vec::new();
        if self.fur_settings.show_chart_total_time_box && self.report.total_time > 0 {
            timer_earnings_boxes_widgets.push(
                column![
                    text(seconds_to_formatted_duration(self.report.total_time, true)).size(50),
                    text(self.localization.get_message("total-time", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }
        if self.fur_settings.show_chart_total_earnings_box && self.report.total_earned > 0.0 {
            timer_earnings_boxes_widgets.push(
                column![
                    text!("${:.2}", self.report.total_earned).size(50),
                    text(self.localization.get_message("earned", None)),
                ]
                .align_x(Alignment::Center)
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

            charts_column = charts_column.push(
                Row::with_children(timer_earnings_boxes_widgets).padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 10.0,
                    left: 0.0,
                }),
            );
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
        let mut selection_timer_earnings_boxes_widgets: Vec<Element<'_, Message, Theme, Renderer>> =
            Vec::new();
        if self.fur_settings.show_chart_total_time_box && self.report.selection_total_time > 0 {
            selection_timer_earnings_boxes_widgets.push(
                column![
                    text(seconds_to_formatted_duration(
                        self.report.selection_total_time,
                        true
                    ))
                    .size(50),
                    text(self.localization.get_message("total-time", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }
        if self.fur_settings.show_chart_total_earnings_box
            && self.report.selection_total_earned > 0.0
        {
            selection_timer_earnings_boxes_widgets.push(
                column![
                    text!("${:.2}", self.report.selection_total_earned).size(50),
                    text(self.localization.get_message("earned", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }

        let mut charts_breakdown_by_selection_column = Column::new().align_x(Alignment::Center);
        if !self.report.tasks_in_range.is_empty()
            && self.fur_settings.show_chart_breakdown_by_selection
        {
            charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                text(
                    self.localization
                        .get_message("breakdown-by-selection", None),
                )
                .size(40),
            );
            charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                row![
                    pick_list(
                        &FurTaskProperty::ALL[..],
                        self.report.picked_task_property_key,
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

            if !selection_timer_earnings_boxes_widgets.is_empty() {
                // If both boxes are present, place a spacer between them
                if selection_timer_earnings_boxes_widgets.len() == 2 {
                    selection_timer_earnings_boxes_widgets
                        .insert(1, horizontal_space().width(Length::Fill).into());
                }
                // Then place the bookend spacers
                selection_timer_earnings_boxes_widgets
                    .insert(0, horizontal_space().width(Length::Fill).into());
                selection_timer_earnings_boxes_widgets
                    .push(horizontal_space().width(Length::Fill).into());

                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                    Row::with_children(selection_timer_earnings_boxes_widgets).padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 10.0,
                        left: 0.0,
                    }),
                );
            }

            if self.fur_settings.show_chart_selection_time {
                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column
                    .push(self.report.selection_time_recorded_chart.view());
            }
            if self.fur_settings.show_chart_selection_earnings {
                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column
                    .push(self.report.selection_earnings_recorded_chart.view());
            }
        };

        let charts_view = column![
            column![
                pick_list(
                    &FurDateRange::ALL[..],
                    self.report.picked_date_range,
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
                                .on_press(Message::ChooseStartDate)
                                .style(style::primary_button_style),
                            Message::CancelStartDate,
                            Message::SubmitStartDate,
                        ),
                        column![text("to")
                            .align_y(alignment::Vertical::Center)
                            .height(Length::Fill),]
                        .height(30),
                        date_picker(
                            self.report.show_end_date_picker,
                            self.report.picked_end_date,
                            button(text(self.report.picked_end_date.to_string()))
                                .on_press(Message::ChooseEndDate)
                                .style(style::primary_button_style),
                            Message::CancelEndDate,
                            Message::SubmitEndDate,
                        ),
                        horizontal_space().width(Length::Fill),
                    ]
                    .spacing(30)
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                } else {
                    row![]
                },
                vertical_space().height(Length::Fixed(20.0)),
                horizontal_rule(1),
            ]
            .padding(Padding {
                top: 20.0,
                right: 20.0,
                bottom: 10.0,
                left: 20.0,
            }),
            Scrollable::new(
                column![charts_column, charts_breakdown_by_selection_column]
                    .align_x(Alignment::Center)
                    .padding(Padding {
                        top: 0.0,
                        right: 20.0,
                        bottom: 20.0,
                        left: 20.0,
                    })
            ),
        ];

        // TODO: Change to tabbed report view once iced has lists
        // let report_view: Column<'_, Message, Theme, Renderer> =
        //     column![Tabs::new(Message::ReportTabSelected)
        //         .tab_icon_position(iced_aw::tabs::Position::Top)
        //         .push(
        //             TabId::Charts,
        //             TabLabel::IconText(
        //                 icon_to_char(Bootstrap::GraphUp),
        //                 "Charts".to_string()
        //             ),
        //             charts_view,
        //         )
        //         .push(
        //             TabId::List,
        //             TabLabel::IconText(
        //                 icon_to_char(Bootstrap::ListNested),
        //                 "List".to_string()
        //             ),
        //             Scrollable::new(column![].padding(10)),
        //         )
        //         .set_active_tab(&self.report.active_tab)
        //         .tab_bar_position(TabBarPosition::Top)];

        // MARK: SETTINGS
        let mut server_choice_col = column![pick_list(
            &ServerChoices::ALL[..],
            self.settings_server_choice,
            Message::SettingsServerChoiceSelected,
        ),]
        .padding([0, 15])
        .spacing(10);
        server_choice_col = server_choice_col.push_maybe(
            if self.settings_server_choice == Some(ServerChoices::Custom) {
                Some(
                    text_input("", &self.fur_user_fields.server)
                        .on_input(Message::UserServerChanged)
                        .on_submit(Message::EnterPressedInSyncFields),
                )
            } else {
                None
            },
        );

        let mut sync_server_col = column![
            row![
                text(self.localization.get_message("server", None)),
                server_choice_col,
            ]
            .align_y(Alignment::Center),
            row![
                text(self.localization.get_message("email", None)),
                column![text_input("", &self.fur_user_fields.email)
                    .on_input(Message::UserEmailChanged)
                    .on_submit(Message::EnterPressedInSyncFields)]
                .padding([0, 15])
            ]
            .align_y(Alignment::Center),
            row![
                text(self.localization.get_message("encryption-key", None)),
                column![text_input("", &self.fur_user_fields.encryption_key)
                    .secure(true)
                    .on_input(Message::UserEncryptionKeyChanged)
                    .on_submit(Message::EnterPressedInSyncFields)]
                .padding([0, 15])
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(10);
        let mut sync_button_row: Row<'_, Message> =
            row![button(text(self.localization.get_message(
                if self.fur_user.is_none() {
                    "log-in"
                } else {
                    "log-out"
                },
                None
            )))
            .on_press_maybe(if self.fur_user.is_none() {
                if !self.fur_user_fields.server.is_empty()
                    && !self.fur_user_fields.email.is_empty()
                    && !self.fur_user_fields.encryption_key.is_empty()
                {
                    Some(Message::UserLoginPressed)
                } else {
                    None
                }
            } else {
                Some(Message::UserLogoutPressed)
            })
            .style(if self.fur_user.is_none() {
                style::primary_button_style
            } else {
                button::secondary
            }),]
            .spacing(10);
        sync_button_row = sync_button_row.push_maybe(if self.fur_user.is_some() {
            Some(
                button(text(self.localization.get_message("sync", None)))
                    .on_press_maybe(match self.fur_user {
                        Some(_) => Some(Message::SyncWithServer),
                        None => None,
                    })
                    .style(style::primary_button_style),
            )
        } else {
            Some(
                button(text(self.localization.get_message("sign-up", None)))
                    .on_press(Message::OpenUrl("https://furtherance.app/sync".to_string()))
                    .style(style::primary_button_style),
            )
        });
        sync_server_col = sync_server_col.push(sync_button_row);
        sync_server_col = sync_server_col.push_maybe(match &self.login_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let mut database_location_col = column![
            text(self.localization.get_message("database-location", None)),
            text_input(
                &self.fur_settings.database_url,
                &self.fur_settings.database_url,
            ),
            row![
                button(text(self.localization.get_message("create-new", None)))
                    .on_press(Message::SettingsChangeDatabaseLocationPressed(
                        ChangeDB::New
                    ))
                    .style(style::primary_button_style),
                button(text(self.localization.get_message("open-existing", None)))
                    .on_press(Message::SettingsChangeDatabaseLocationPressed(
                        ChangeDB::Open
                    ))
                    .style(style::primary_button_style),
                button(text(self.localization.get_message("backup-database", None)))
                    .on_press(Message::BackupDatabase)
                    .style(style::primary_button_style)
            ]
            .spacing(10)
            .wrap(),
        ]
        .spacing(10);
        database_location_col =
            database_location_col.push_maybe(match &self.settings_database_message {
                Ok(msg) => {
                    if msg.is_empty() {
                        None
                    } else {
                        Some(text(msg).style(style::green_text))
                    }
                }
                Err(e) => Some(text!("{}", e).style(style::red_text)),
            });

        let mut csv_col = column![row![
            button(text(self.localization.get_message("export-csv", None)))
                .on_press(Message::ExportCsvPressed)
                .style(style::primary_button_style),
            button(text(self.localization.get_message("import-csv", None)))
                .on_press(Message::ImportCsvPressed)
                .style(style::primary_button_style)
        ]
        .spacing(10),]
        .spacing(10);
        csv_col = csv_col.push_maybe(match &self.settings_csv_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let mut backup_col = column![button(text(
            self.localization.get_message("delete-everything", None)
        ))
        .on_press(Message::ShowAlert(FurAlert::DeleteEverythingConfirmation))
        .style(button::danger)]
        .spacing(10);
        backup_col = backup_col.push_maybe(match &self.settings_more_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let settings_view: Column<'_, Message, Theme, Renderer> =
            column![Tabs::new(Message::SettingsTabSelected)
                .tab_icon_position(iced_aw::tabs::Position::Top)
                .push(
                    TabId::General,
                    TabLabel::IconText(
                        icon_to_char(Bootstrap::GearFill),
                        self.localization.get_message("general", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("interface", None)),
                            row![
                                text(self.localization.get_message("default-view", None)),
                                pick_list(
                                    &FurView::ALL[..],
                                    Some(self.fur_settings.default_view),
                                    Message::SettingsDefaultViewSelected,
                                ),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("theme", None)),
                                pick_list(
                                    &FurDarkLight::ALL[..],
                                    Some(self.fur_settings.theme),
                                    Message::SettingsThemeSelected,
                                ),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            if cfg!(target_os = "macos")
                                && self.fur_settings.theme == FurDarkLight::Auto
                            {
                                row![
                                    text(self.localization.get_message("mac-theme-warning", None))
                                        .style(style::red_text)
                                ]
                            } else {
                                row![].height(Length::Shrink)
                            },
                            row![
                                text(
                                    self.localization
                                        .get_message("show-delete-confirmation", None)
                                ),
                                toggler(self.fur_settings.show_delete_confirmation)
                                    .on_toggle(Message::SettingsDeleteConfirmationToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("task-history", None)),
                            row![
                                text(self.localization.get_message("show-project", None)),
                                toggler(self.fur_settings.show_project)
                                    .on_toggle(Message::SettingsShowProjectToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-tags", None)),
                                toggler(self.fur_settings.show_tags)
                                    .on_toggle(Message::SettingsShowTagsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-earnings", None)),
                                toggler(self.fur_settings.show_earnings)
                                    .on_toggle(Message::SettingsShowEarningsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-seconds", None)),
                                toggler(self.fur_settings.show_seconds)
                                    .on_toggle(Message::SettingsShowSecondsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-daily-time-total", None)),
                                toggler(self.fur_settings.show_daily_time_total)
                                    .on_toggle(Message::SettingsShowDailyTimeTotalToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10)
                    ),
                )
                .push(
                    TabId::Advanced,
                    TabLabel::IconText(
                        icon_to_char(Bootstrap::GearWideConnected),
                        self.localization.get_message("advanced", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("idle", None)),
                            row![
                                text(self.localization.get_message("idle-detection", None)),
                                toggler(self.fur_settings.notify_on_idle)
                                    .on_toggle(Message::SettingsIdleToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("minutes-until-idle", None)),
                                number_input(
                                    self.fur_settings.chosen_idle_time,
                                    1..999,
                                    Message::SettingsIdleTimeChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("task-history", None)),
                            row![
                                column![
                                    text(self.localization.get_message("dynamic-total", None)),
                                    text(
                                        self.localization
                                            .get_message("dynamic-total-description", None)
                                    )
                                    .size(12),
                                ],
                                toggler(self.fur_settings.dynamic_total)
                                    .on_toggle(Message::SettingsDynamicTotalToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("days-to-show", None)),
                                number_input(
                                    self.fur_settings.days_to_show,
                                    1..=365,
                                    Message::SettingsDaysToShowChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(
                                self.localization.get_message("reminder-notification", None)
                            ),
                            row![
                                column![
                                    text(
                                        self.localization
                                            .get_message("reminder-notifications", None)
                                    ),
                                    text(
                                        self.localization.get_message(
                                            "reminder-notifications-description",
                                            None
                                        )
                                    )
                                    .size(12),
                                ],
                                toggler(self.fur_settings.notify_reminder)
                                    .on_toggle(Message::SettingsRemindersToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("reminder-interval", None)),
                                number_input(
                                    self.fur_settings.notify_reminder_interval,
                                    1..999,
                                    Message::SettingsReminderIntervalChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![text(format!("Furtherance version {}", FURTHERANCE_VERSION))
                                .font(font::Font {
                                    style: iced::font::Style::Italic,
                                    ..Default::default()
                                })]
                            .padding(Padding {
                                top: 40.0,
                                right: 0.0,
                                bottom: 0.0,
                                left: 0.0,
                            }),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10)
                    ),
                )
                .push(
                    TabId::Pomodoro,
                    TabLabel::IconText(
                        icon_to_char(Bootstrap::StopwatchFill),
                        self.localization.get_message("pomodoro", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("pomodoro-timer", None)),
                            row![
                                text(self.localization.get_message("notification_alarm_sound", None)),
                                toggler(self.fur_settings.pomodoro_notification_alarm_sound)
                                    .on_toggle(Message::SettingsPomodoroNotificationAlarmSoundToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("countdown-timer", None)),
                                toggler(self.fur_settings.pomodoro)
                                    .on_toggle_maybe(if self.timer_is_running {
                                        None
                                    } else {
                                        Some(Message::SettingsPomodoroToggled)
                                    })
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("timer-length", None)),
                                number_input(
                                    self.fur_settings.pomodoro_length,
                                    1..999,
                                    Message::SettingsPomodoroLengthChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("break-length", None)),
                                number_input(
                                    self.fur_settings.pomodoro_break_length,
                                    1..999,
                                    Message::SettingsPomodoroBreakLengthChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("snooze-length", None)),
                                number_input(
                                    self.fur_settings.pomodoro_snooze_length,
                                    1..999,
                                    Message::SettingsPomodoroSnoozeLengthChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("extended-break", None)),
                            row![
                                text(self.localization.get_message("extended-breaks", None)),
                                toggler(self.fur_settings.pomodoro_extended_breaks)
                                    .on_toggle(Message::SettingsPomodoroExtendedBreaksToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(
                                    self.localization
                                        .get_message("extended-break-interval", None)
                                ),
                                number_input(
                                    self.fur_settings.pomodoro_extended_break_interval,
                                    1..999,
                                    Message::SettingsPomodoroExtendedBreakIntervalChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("extended-break-length", None)),
                                number_input(
                                    self.fur_settings.pomodoro_extended_break_length,
                                    1..999,
                                    Message::SettingsPomodoroExtendedBreakLengthChanged
                                )
                                .width(Length::Shrink)
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                .push(
                    TabId::Report,
                    TabLabel::IconText(
                        icon_to_char(Bootstrap::GraphUp),
                        self.localization.get_message("report", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("toggle-charts", None)),
                            checkbox(
                                self.localization.get_message("total-time-box", None),
                                self.fur_settings.show_chart_total_time_box
                            )
                            .on_toggle(Message::SettingsShowChartTotalTimeBoxToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization.get_message("total-earnings-box", None),
                                self.fur_settings.show_chart_total_earnings_box
                            )
                            .on_toggle(Message::SettingsShowChartTotalEarningsBoxToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization.get_message("time-recorded", None),
                                self.fur_settings.show_chart_time_recorded
                            )
                            .on_toggle(Message::SettingsShowChartTimeRecordedToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization.get_message("earnings", None),
                                self.fur_settings.show_chart_earnings
                            )
                            .on_toggle(Message::SettingsShowChartEarningsToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization.get_message("average-time-per-task", None),
                                self.fur_settings.show_chart_average_time
                            )
                            .on_toggle(Message::SettingsShowChartAverageTimeToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization
                                    .get_message("average-earnings-per-task", None),
                                self.fur_settings.show_chart_average_earnings
                            )
                            .on_toggle(Message::SettingsShowChartAverageEarningsToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization
                                    .get_message("breakdown-by-selection-section", None),
                                self.fur_settings.show_chart_breakdown_by_selection
                            )
                            .on_toggle(Message::SettingsShowChartBreakdownBySelectionToggled)
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization
                                    .get_message("time-recorded-for-selection", None),
                                self.fur_settings.show_chart_selection_time
                            )
                            .on_toggle_maybe(
                                if self.fur_settings.show_chart_breakdown_by_selection {
                                    Some(Message::SettingsShowChartSelectionTimeToggled)
                                } else {
                                    None
                                }
                            )
                            .style(style::fur_checkbox_style),
                            checkbox(
                                self.localization
                                    .get_message("earnings-for-selection", None),
                                self.fur_settings.show_chart_selection_earnings
                            )
                            .on_toggle_maybe(
                                if self.fur_settings.show_chart_breakdown_by_selection {
                                    Some(Message::SettingsShowChartSelectionEarningsToggled)
                                } else {
                                    None
                                }
                            )
                            .style(style::fur_checkbox_style),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                // MARK: SETTINGS DATA TAB
                .push(
                    TabId::Data,
                    TabLabel::IconText(
                        icon_to_char(Bootstrap::FloppyFill),
                        self.localization.get_message("data", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("sync", None)),
                            sync_server_col,
                            settings_heading(self.localization.get_message("local-database", None)),
                            database_location_col,
                            settings_heading("CSV".to_string()),
                            csv_col,
                            settings_heading(self.localization.get_message("more", None)),
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
                    text_input(
                        &self.localization.get_message("task-name", None),
                        &task_to_add.name
                    )
                    .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Name)),
                    text_input(
                        &self.localization.get_message("project", None),
                        &task_to_add.project
                    )
                    .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Project)),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &task_to_add.tags
                    )
                    .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags)),
                    text_input("0.00", &task_to_add.new_rate)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Rate)),
                    row![
                        text(self.localization.get_message("start-colon", None)),
                        date_picker(
                            task_to_add.show_start_date_picker,
                            task_to_add.displayed_start_date,
                            button(text(task_to_add.displayed_start_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StartDate),
                        ),
                        time_picker(
                            task_to_add.show_start_time_picker,
                            task_to_add.displayed_start_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_start_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StartTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("stop-colon", None)),
                        date_picker(
                            task_to_add.show_stop_date_picker,
                            task_to_add.displayed_stop_date,
                            button(text(task_to_add.displayed_stop_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StopDate),
                        ),
                        time_picker(
                            task_to_add.show_stop_time_picker,
                            task_to_add.displayed_stop_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_stop_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StopTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTaskEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::primary)
                        .on_press_maybe(if task_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveTaskEdit)
                        })
                        .width(Length::Fill)
                        .style(style::primary_button_style),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            // Add shortcut
            Some(FurInspectorView::AddShortcut) => match &self.shortcut_to_add {
                Some(shortcut_to_add) => column![
                    text(self.localization.get_message("new-shortcut", None)).size(24),
                    text_input(
                        &self.localization.get_message("task-name", None),
                        &shortcut_to_add.name
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Name)),
                    text_input(
                        &self.localization.get_message("project", None),
                        &shortcut_to_add.project
                    )
                    .on_input(|s| {
                        Message::EditShortcutTextChanged(s, EditTaskProperty::Project)
                    }),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &shortcut_to_add.tags
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Tags)),
                    row![
                        text("$"),
                        text_input("0.00", &shortcut_to_add.new_rate).on_input(|s| {
                            Message::EditShortcutTextChanged(s, EditTaskProperty::Rate)
                        }),
                        text(self.localization.get_message("per-hour", None)),
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center),
                    color_picker(
                        shortcut_to_add.show_color_picker,
                        shortcut_to_add.color,
                        button(
                            text(self.localization.get_message("color", None))
                                .style(|_| if is_dark_color(shortcut_to_add.color.to_srgb()) {
                                    text::Style {
                                        color: Some(Color::WHITE),
                                    }
                                } else {
                                    text::Style {
                                        color: Some(Color::BLACK),
                                    }
                                })
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press(Message::ChooseShortcutColor)
                        .width(Length::Fill)
                        .style(|theme, status| {
                            style::shortcut_button_style(
                                theme,
                                status,
                                shortcut_to_add.color.to_srgb(),
                            )
                        }),
                        Message::CancelShortcutColor,
                        Message::SubmitShortcutColor,
                    ),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelShortcut)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press_maybe(if shortcut_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        })
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&shortcut_to_add.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            Some(FurInspectorView::AddTaskToGroup) => match &self.task_to_add {
                Some(task_to_add) => column![
                    text_input(&task_to_add.name, ""),
                    text_input(&task_to_add.project, ""),
                    text_input(&task_to_add.tags, ""),
                    text_input(&format!("{:.2}", task_to_add.rate), ""),
                    row![
                        text(self.localization.get_message("start-colon", None)),
                        button(
                            text(task_to_add.displayed_start_date.to_string())
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press_maybe(None)
                        .style(style::primary_button_style),
                        time_picker(
                            task_to_add.show_start_time_picker,
                            task_to_add.displayed_start_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_start_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StartTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("stop-colon", None)),
                        button(
                            text(task_to_add.displayed_stop_date.to_string())
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press_maybe(None)
                        .style(style::primary_button_style),
                        time_picker(
                            task_to_add.show_stop_time_picker,
                            task_to_add.displayed_stop_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_stop_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StopTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTaskEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press(Message::SaveTaskEdit)
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            // MARK: Edit Shortcut
            Some(FurInspectorView::EditShortcut) => match &self.shortcut_to_edit {
                Some(shortcut_to_edit) => column![
                    text(self.localization.get_message("edit-shortcut", None)).size(24),
                    text_input(
                        &self.localization.get_message("task-name", None),
                        &shortcut_to_edit.new_name
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Name)),
                    text_input(
                        &self.localization.get_message("project", None),
                        &shortcut_to_edit.new_project
                    )
                    .on_input(|s| {
                        Message::EditShortcutTextChanged(s, EditTaskProperty::Project)
                    }),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &shortcut_to_edit.new_tags
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Tags)),
                    row![
                        text("$"),
                        text_input("0.00", &shortcut_to_edit.new_rate).on_input(|s| {
                            Message::EditShortcutTextChanged(s, EditTaskProperty::Rate)
                        }),
                        text(self.localization.get_message("per-hour", None)),
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center),
                    color_picker(
                        shortcut_to_edit.show_color_picker,
                        shortcut_to_edit.new_color,
                        button(
                            text(self.localization.get_message("color", None))
                                .style(|_| if is_dark_color(shortcut_to_edit.new_color.to_srgb()) {
                                    text::Style {
                                        color: Some(Color::WHITE),
                                    }
                                } else {
                                    text::Style {
                                        color: Some(Color::BLACK),
                                    }
                                })
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press(Message::ChooseShortcutColor)
                        .width(Length::Fill)
                        .style(|theme, status| {
                            style::shortcut_button_style(
                                theme,
                                status,
                                shortcut_to_edit.new_color.to_srgb(),
                            )
                        }),
                        Message::CancelShortcutColor,
                        Message::SubmitShortcutColor,
                    ),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelShortcut)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
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
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&shortcut_to_edit.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(INSPECTOR_SPACING)
                    .padding(INSPECTOR_PADDING)
                    .width(INSPECTOR_WIDTH)
                    .align_x(INSPECTOR_ALIGNMENT),
            },
            // MARK: Edit Single Task
            Some(FurInspectorView::EditTask) => match &self.task_to_edit {
                Some(task_to_edit) => column![
                    row![
                        horizontal_space(),
                        button(text(icon_to_char(Bootstrap::TrashFill)).font(BOOTSTRAP_FONT))
                            .on_press(if self.fur_settings.show_delete_confirmation {
                                Message::ShowAlert(FurAlert::DeleteTaskConfirmation)
                            } else {
                                Message::DeleteTasks
                            })
                            .style(button::text),
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
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("start-colon", None)),
                        date_picker(
                            task_to_edit.show_displayed_start_date_picker,
                            task_to_edit.displayed_start_date,
                            button(text(task_to_edit.displayed_start_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StartDate),
                        ),
                        time_picker(
                            task_to_edit.show_displayed_start_time_picker,
                            task_to_edit.displayed_start_time,
                            Button::new(
                                text(format_iced_time_as_hm(task_to_edit.displayed_start_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StartTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("stop-colon", None)),
                        date_picker(
                            task_to_edit.show_displayed_stop_date_picker,
                            task_to_edit.displayed_stop_date,
                            button(text(task_to_edit.displayed_stop_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StopDate),
                        ),
                        time_picker(
                            task_to_edit.show_displayed_stop_time_picker,
                            task_to_edit.displayed_stop_time,
                            button(
                                text(format_iced_time_as_hm(task_to_edit.displayed_stop_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StopTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTaskEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press_maybe(
                            if task_to_edit.is_changed() && !task_to_edit.new_name.trim().is_empty()
                            {
                                Some(Message::SaveTaskEdit)
                            } else {
                                None
                            }
                        )
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&task_to_edit.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![].width(INSPECTOR_WIDTH),
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
                        .align_x(Alignment::Center)
                        .spacing(5)
                        .padding(20);
                    if !group_to_edit.project.is_empty() {
                        group_info_column = group_info_column.push(text(&group_to_edit.project));
                    }
                    if !group_to_edit.tags.is_empty() {
                        group_info_column =
                            group_info_column.push(text!("#{}", group_to_edit.tags));
                    }
                    if group_to_edit.rate != 0.0 {
                        group_info_column =
                            group_info_column.push(text!("${}", &group_to_edit.rate));
                    }
                    let tasks_column: Scrollable<'_, Message, Theme, Renderer> =
                        Scrollable::new(group_to_edit.tasks.iter().fold(
                            Column::new().spacing(5),
                            |column, task| {
                                column
                                    .push(
                                        button(
                                            Container::new(column![
                                                text(
                                                    self.localization.get_message(
                                                        "start-to-stop",
                                                        Some(&HashMap::from([
                                                            (
                                                                "start",
                                                                FluentValue::from(
                                                                    task.start_time
                                                                        .format("%H:%M")
                                                                        .to_string()
                                                                )
                                                            ),
                                                            (
                                                                "stop",
                                                                FluentValue::from(
                                                                    task.stop_time
                                                                        .format("%H:%M")
                                                                        .to_string()
                                                                )
                                                            )
                                                        ]))
                                                    )
                                                )
                                                .font(font::Font {
                                                    weight: iced::font::Weight::Bold,
                                                    ..Default::default()
                                                }),
                                                text(self.localization.get_message(
                                                    "total-time-dynamic",
                                                    Some(&HashMap::from([(
                                                        "time",
                                                        FluentValue::from(
                                                            seconds_to_formatted_duration(
                                                                task.total_time_in_seconds(),
                                                                self.fur_settings.show_seconds
                                                            )
                                                        )
                                                    )]))
                                                ))
                                            ])
                                            .width(Length::Fill)
                                            .padding([5, 8])
                                            .style(style::group_edit_task_row),
                                        )
                                        .on_press(Message::EditTask(task.clone()))
                                        .style(button::text),
                                    )
                                    .padding(Padding {
                                        top: 0.0,
                                        right: 10.0,
                                        bottom: 10.0,
                                        left: 10.0,
                                    })
                            },
                        ));
                    column![
                        row![
                            button(text(icon_to_char(Bootstrap::XLg)).font(BOOTSTRAP_FONT))
                                .on_press(Message::CancelGroupEdit)
                                .style(button::text),
                            horizontal_space(),
                            button(if group_to_edit.is_in_edit_mode {
                                text(icon_to_char(Bootstrap::Pencil)).font(BOOTSTRAP_FONT)
                            } else {
                                text(icon_to_char(Bootstrap::PencilFill)).font(BOOTSTRAP_FONT)
                            })
                            .on_press_maybe(if group_to_edit.is_in_edit_mode {
                                None
                            } else {
                                Some(Message::ToggleGroupEditor)
                            })
                            .style(button::text),
                            button(text(icon_to_char(Bootstrap::PlusLg)).font(BOOTSTRAP_FONT))
                                .on_press_maybe(if group_to_edit.is_in_edit_mode {
                                    None
                                } else {
                                    Some(Message::AddTaskToGroup(group_to_edit.clone()))
                                })
                                .style(button::text),
                            button(text(icon_to_char(Bootstrap::TrashFill)).font(BOOTSTRAP_FONT))
                                .on_press(if self.fur_settings.show_delete_confirmation {
                                    Message::ShowAlert(FurAlert::DeleteGroupConfirmation)
                                } else {
                                    Message::DeleteTasks
                                })
                                .style(button::text),
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
                                .align_y(Alignment::Center)
                                .spacing(5),
                                row![
                                    button(
                                        text(self.localization.get_message("cancel", None))
                                            .align_x(alignment::Horizontal::Center)
                                    )
                                    .style(button::secondary)
                                    .on_press(Message::ToggleGroupEditor)
                                    .width(Length::Fill),
                                    button(
                                        text(self.localization.get_message("save", None))
                                            .align_x(alignment::Horizontal::Center)
                                    )
                                    .style(style::primary_button_style)
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
                                .padding(Padding {
                                    top: 20.0,
                                    right: 0.0,
                                    bottom: 0.0,
                                    left: 0.0,
                                })
                                .spacing(10),
                            ]
                            .padding(20)
                            .spacing(5),
                            false => group_info_column,
                        },
                        tasks_column,
                    ]
                    .spacing(5)
                    .align_x(Alignment::Start)
                }
                None => column![text(
                    self.localization.get_message("nothing-selected", None)
                )]
                .spacing(12)
                .padding(20)
                .align_x(Alignment::Start),
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
            let alert_description: String;
            let close_button: Option<Button<'_, Message, Theme, Renderer>>;
            let mut confirmation_button: Option<Button<'_, Message, Theme, Renderer>> = None;
            let mut snooze_button: Option<Button<'_, Message, Theme, Renderer>> = None;

            match self.displayed_alert.as_ref().unwrap() {
                FurAlert::AutosaveRestored => {
                    alert_text = self.localization.get_message("autosave-restored", None);
                    alert_description = self
                        .localization
                        .get_message("autosave-restored-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("ok", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::primary),
                    );
                }
                FurAlert::DeleteEverythingConfirmation => {
                    alert_text = self
                        .localization
                        .get_message("delete-everything-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-everything-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete-everything", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteEverything)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteGroupConfirmation => {
                    alert_text = self.localization.get_message("delete-all-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-all-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete-all", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTasks)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteShortcutConfirmation => {
                    alert_text = self
                        .localization
                        .get_message("delete-shortcut-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-shortcut-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteShortcut)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteTaskConfirmation => {
                    alert_text = self.localization.get_message("delete-task-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-task-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTasks)
                        .style(button::danger),
                    );
                }
                FurAlert::Idle => {
                    alert_text = self.localization.get_message(
                        "idle-alert-title",
                        Some(&HashMap::from([(
                            "duration",
                            FluentValue::from(self.idle.duration()),
                        )])),
                    );
                    alert_description = self
                        .localization
                        .get_message("idle-alert-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("continue", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::IdleReset)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("discard", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::IdleDiscard)
                        .style(button::danger),
                    );
                }
                FurAlert::ImportMacDatabase => {
                    alert_text = self.localization.get_message("import-old-database", None);
                    alert_description = self
                        .localization
                        .get_message("import-old-database-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("dont-import", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("import", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::ImportOldMacDatabase)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::NotifyOfSync => {
                    alert_text = self.localization.get_message("syncing-now-available", None);
                    alert_description = self.localization.get_message("syncing-now-possible", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("ok", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::NotifyOfSyncClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("learn-more", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::LearnAboutSync)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::PomodoroBreakOver => {
                    alert_text = self.localization.get_message("break-over-title", None);
                    alert_description = self
                        .localization
                        .get_message("break-over-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("stop", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStopAfterBreak)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("continue", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroContinueAfterBreak)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::PomodoroOver => {
                    alert_text = self.localization.get_message("pomodoro-over-title", None);
                    alert_description = self
                        .localization
                        .get_message("pomodoro-over-description", None);
                    snooze_button = Some(
                        button(
                            text(self.localization.get_message(
                                "snooze-button",
                                Some(&HashMap::from([(
                                    "duration",
                                    FluentValue::from(self.fur_settings.pomodoro_snooze_length),
                                )])),
                            ))
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Shrink),
                        )
                        .on_press(Message::PomodoroSnooze)
                        .style(button::secondary),
                    );
                    close_button = Some(
                        button(
                            text(self.localization.get_message("stop", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStop)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(
                                if self.fur_settings.pomodoro_extended_breaks
                                    && self.pomodoro.sessions
                                        % self.fur_settings.pomodoro_extended_break_interval
                                        == 0
                                {
                                    self.localization.get_message("long-break", None)
                                } else {
                                    self.localization.get_message("break", None)
                                },
                            )
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStartBreak)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::ShortcutExists => {
                    alert_text = self.localization.get_message("shortcut-exists", None);
                    alert_description = self
                        .localization
                        .get_message("shortcut-exists-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("ok", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(style::primary_button_style),
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
                    })
                    .style(style::fur_card),
            )
        } else {
            None
        };

        if let Some(alert) = overlay {
            modal(content, container(alert)).into()
        } else {
            content.into()
        }
    }
}

fn nav_button<'a>(nav_text: String, destination: FurView, active: bool) -> Button<'a, Message> {
    button(text(nav_text))
        .padding([5, 15])
        .on_press(Message::NavigateTo(destination))
        .width(Length::Fill)
        .style(if active {
            style::active_nav_menu_button_style
        } else {
            style::inactive_nav_menu_button_style
        })
}

fn history_group_row<'a, 'loc>(
    task_group: &'a FurTaskGroup,
    timer_is_running: bool,
    settings: &'a FurSettings,
    localization: &'loc Localization,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'loc>, Message> {
    let mut task_details_column: Column<'_, Message, Theme, Renderer> =
        column![text(&task_group.name).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),]
        .width(Length::FillPortion(6));
    if settings.show_project && !task_group.project.is_empty() {
        task_details_column = task_details_column.push(text!("@{}", task_group.project));
    }
    if settings.show_tags && !task_group.tags.is_empty() {
        task_details_column = task_details_column.push(text!("#{}", task_group.tags));
    }

    let mut task_row: Row<'_, Message, Theme, Renderer> =
        row![].align_y(Alignment::Center).spacing(5);
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
    .align_x(Alignment::End);

    if settings.show_earnings && task_group.rate > 0.0 {
        let total_earnings = task_group.rate * (task_group.total_time as f32 / 3600.0);
        totals_column = totals_column.push(text!("${:.2}", total_earnings));
    }

    let task_group_string = task_group.to_string();

    task_row = task_row.push(task_details_column);
    task_row = task_row.push(horizontal_space().width(Length::Fill));
    task_row = task_row.push(totals_column);
    task_row = task_row.push(
        button(text(icon_to_char(Bootstrap::ArrowRepeat)).font(BOOTSTRAP_FONT))
            .on_press_maybe(if timer_is_running {
                None
            } else {
                Some(Message::RepeatLastTaskPressed(task_group_string.clone()))
            })
            .style(button::text),
    );

    let history_row_button = button(
        Container::new(task_row)
            .padding([10, 15])
            .width(Length::Fill)
            .style(style::task_row),
    )
    .on_press(Message::EditGroup(task_group.clone()))
    .style(button::text);

    let task_group_ids = task_group.all_task_ids();
    let task_group_clone = task_group.clone();

    ContextMenu::new(
        history_row_button,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button(text(localization.get_message("repeat", None)))
                    .on_press(Message::RepeatLastTaskPressed(task_group_string.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("edit", None)))
                    .on_press(Message::EditGroup(task_group_clone.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("create-shortcut", None)))
                    .on_press(Message::CreateShortcutFromTaskGroup(
                        task_group_clone.clone(),
                    ))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("delete", None)))
                    .on_press(Message::DeleteTasksFromContext(task_group_ids.clone()))
                    .style(style::context_menu_button_style)
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
    localization: &Localization,
) -> Row<'a, Message> {
    let mut total_time_column = column![].align_x(Alignment::End);

    if settings.show_daily_time_total {
        if settings.dynamic_total {
            if let Some((running, timer_text)) = running_timer {
                if running {
                    let total_time_str = seconds_to_formatted_duration(
                        combine_timer_with_seconds(timer_text, total_time),
                        settings.show_seconds,
                    );
                    total_time_column =
                        total_time_column.push(text(total_time_str).font(font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }));
                } else {
                    let total_time_str =
                        seconds_to_formatted_duration(total_time, settings.show_seconds);
                    total_time_column =
                        total_time_column.push(text(total_time_str).font(font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }));
                }
            } else {
                let total_time_str =
                    seconds_to_formatted_duration(total_time, settings.show_seconds);
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
        total_time_column = total_time_column.push(text!("${:.2}", total_earnings));
    }

    row![
        text(format_history_date(date, localization)).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        horizontal_space().width(Length::Fill),
        total_time_column,
    ]
    .align_y(Alignment::Center)
}

fn format_history_date(date: &NaiveDate, localization: &Localization) -> String {
    let today = Local::now().date_naive();
    let yesterday = today - TimeDelta::days(1);
    let current_year = today.year();

    if date == &today {
        localization.get_message("today", None)
    } else if date == &yesterday {
        localization.get_message("yesterday", None)
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
    shortcut: &'a FurShortcut,
    text_color: Color,
) -> Column<'a, Message, Theme, Renderer> {
    let mut shortcut_text_column = column![text(&shortcut.name)
        .font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .style(move |_| text::Style {
            color: Some(text_color)
        })]
    .spacing(5);

    if !shortcut.project.is_empty() {
        shortcut_text_column =
            shortcut_text_column.push(text!("@{}", shortcut.project).style(move |_| text::Style {
                color: Some(text_color),
            }));
    }
    if !shortcut.tags.is_empty() {
        shortcut_text_column =
            shortcut_text_column.push(text(&shortcut.tags).style(move |_| text::Style {
                color: Some(text_color),
            }));
    }
    if shortcut.rate > 0.0 {
        shortcut_text_column = shortcut_text_column.push(vertical_space());
        shortcut_text_column = shortcut_text_column.push(row![
            horizontal_space(),
            text!("${:.2}", shortcut.rate).style(move |_| text::Style {
                color: Some(text_color)
            })
        ]);
    }

    shortcut_text_column
}

fn shortcut_button<'a, 'loc>(
    shortcut: &'a FurShortcut,
    timer_is_running: bool,
    localization: &'loc Localization,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'loc>, Message> {
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
        .style(move |theme, status| style::shortcut_button_style(theme, status, shortcut_color));

    let shortcut_clone = shortcut.clone();

    ContextMenu::new(
        shortcut_button,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button(text(localization.get_message("edit", None)))
                    .on_press(Message::EditShortcutPressed(shortcut_clone.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("delete", None)))
                    .on_press(Message::DeleteShortcutFromContext(
                        shortcut_clone.uid.clone()
                    ))
                    .style(style::context_menu_button_style)
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
    state.displayed_task_start_time = convert_datetime_to_iced_time(state.timer_start_time);
    state.timer_is_running = true;
    if state.fur_settings.pomodoro && !state.pomodoro.on_break {
        state.pomodoro.sessions += 1;
    }

    #[cfg(target_os = "linux")]
    if idle::is_kde() {
        if let Err(e) = wayland_idle::start_idle_monitor() {
            eprintln!("Failed to start idle monitor: {}", e);
        }
    }
}

fn stop_timer(state: &mut Furtherance, stop_time: DateTime<Local>) {
    state.timer_is_running = false;

    let (name, project, tags, rate) = split_task_input(&state.task_input);
    db_insert_task(&FurTask::new(
        name,
        state.timer_start_time,
        stop_time,
        tags,
        project,
        rate,
        String::new(),
    ))
    .expect("Couldn't write task to database.");

    delete_autosave();
    reset_timer(state);

    #[cfg(target_os = "linux")]
    if idle::is_kde() {
        wayland_idle::stop_idle_monitor();
    }
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

fn convert_timer_text_to_vertical_hms(timer_text: &str, localization: &Localization) -> String {
    let mut split = timer_text.split(':');
    let mut sidebar_timer_text = String::new();

    if let Some(hours) = split.next() {
        if hours != "0" {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-h",
                Some(&HashMap::from([("hours", FluentValue::from(hours))])),
            ));
            sidebar_timer_text.push_str("\n");
        }
    }

    if let Some(mins) = split.next() {
        if mins != "00" {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-m",
                Some(&HashMap::from([(
                    "minutes",
                    FluentValue::from(mins.trim_start_matches('0')),
                )])),
            ));
            sidebar_timer_text.push_str("\n");
        }
    }

    if let Some(secs) = split.next() {
        if secs != "00" {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-s",
                Some(&HashMap::from([(
                    "seconds",
                    FluentValue::from(secs.trim_start_matches('0')),
                )])),
            ));
        } else {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-s",
                Some(&HashMap::from([("seconds", FluentValue::from("0"))])),
            ));
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

fn convert_datetime_to_iced_time(dt: DateTime<Local>) -> time_picker::Time {
    time_picker::Time::from(dt.time())
}

async fn get_timer_duration() {
    time::sleep(Duration::from_secs(1)).await;
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

    let tags = re_tags
        .captures_iter(input)
        .map(|cap| cap.get(1).unwrap().as_str().trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .sorted()
        .unique()
        .collect::<Vec<String>>()
        .join(" #");

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

fn show_notification(notification_type: NotificationType, localization: &Localization, notification_alarm_sound: bool) {
    let heading: String;
    let details: String;
    let has_sound: bool;

    match notification_type {
        NotificationType::PomodoroOver => {
            heading = localization.get_message("pomodoro-over-title", None);
            details = localization.get_message("pomodoro-over-notification-body", None);
            has_sound = true;
        }
        NotificationType::BreakOver => {
            heading = localization.get_message("break-over-title", None);
            details = localization.get_message("break-over-description", None);
            has_sound = true;
        }
        NotificationType::Idle => {
            heading = localization.get_message("idle-notification-title", None);
            details = localization.get_message("idle-notification-body", None);
            has_sound = false;
        }
        NotificationType::Reminder => {
            heading = localization.get_message("track-your-time", None);
            details = localization.get_message("did-you-forget", None);
            has_sound = false;
        }
    }

    match Notification::new()
        .summary(&heading)
        .body(&details)
        .appname("Furtherance")
        .sound_name(if has_sound && notification_alarm_sound {"alarm-clock-elapsed"} else {""})
        .timeout(Timeout::Milliseconds(6000))
        .show()
    {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to show notification: {e}"),
    }
}

fn settings_heading<'a>(heading: String) -> Column<'a, Message, Theme, Renderer> {
    column![
        text(heading).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Container::new(horizontal_rule(1)).max_width(200.0)
    ]
    .padding(Padding {
        top: 15.0,
        right: 0.0,
        bottom: 5.0,
        left: 0.0,
    })
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

fn get_system_theme(theme_setting: FurDarkLight) -> FurTheme {
    if theme_setting == FurDarkLight::Auto {
        match dark_light::detect() {
            dark_light::Mode::Light | dark_light::Mode::Default => FurTheme::Light,
            dark_light::Mode::Dark => FurTheme::Dark,
        }
    } else {
        match theme_setting {
            FurDarkLight::Light => FurTheme::Light,
            FurDarkLight::Dark => FurTheme::Dark,
            _ => FurTheme::Light,
        }
    }
}

pub fn write_furtasks_to_csv(
    path: PathBuf,
    localization: &Localization,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(file) = std::fs::File::create(path) {
        if let Ok(tasks) = db_retrieve_all_existing_tasks(SortBy::StartTime, SortOrder::Descending)
        {
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
            Err(localization
                .get_message("error-retrieving-tasks", None)
                .into())
        }
    } else {
        Err(localization.get_message("error-creating-file", None).into())
    }
}

pub fn verify_csv(
    file: &std::fs::File,
    localization: &Localization,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = Reader::from_reader(file);

    // v3 - Iced
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
    // v2 - macOS/SwiftUI
    let v2_headers = vec![
        "Name",
        "Project",
        "Tags",
        "Rate",
        "Start Time",
        "Stop Time",
        "Total Seconds",
    ];
    // v1 - GTK
    let v1_headers = vec![
        "id",
        "task_name",
        "start_time",
        "stop_time",
        "tags",
        "seconds",
    ];

    if let Ok(headers) = rdr.headers() {
        if verify_headers(headers, &v3_headers, localization).is_err() {
            if verify_headers(headers, &v2_headers, localization).is_err() {
                verify_headers(headers, &v1_headers, localization)?;
            }
        }
    } else {
        return Err(localization
            .get_message("error-reading-headers", None)
            .into());
    }

    Ok(())
}

fn verify_headers(
    headers: &StringRecord,
    expected: &[&str],
    localization: &Localization,
) -> Result<(), Box<dyn std::error::Error>> {
    for (i, expected_header) in expected.iter().enumerate() {
        match headers.get(i) {
            Some(header) if header == *expected_header => continue,
            Some(_) => {
                return Err(localization.get_message("wrong-column-order", None).into());
            }
            None => {
                return Err(localization.get_message("missing-column", None).into());
            }
        }
    }
    Ok(())
}

pub fn read_csv(
    file: &File,
    localization: &Localization,
) -> Result<Vec<FurTask>, Box<dyn std::error::Error>> {
    let mut rdr = ReaderBuilder::new().flexible(true).from_reader(file);
    let mut tasks = Vec::new();

    for result in rdr.records() {
        let record = result?;

        let task = match record.len() {
            9 => {
                // v3 - Iced
                FurTask::new_with_last_updated(
                    record.get(0).unwrap_or("").to_string(),
                    record.get(1).unwrap_or("").parse().unwrap_or_default(),
                    record.get(2).unwrap_or("").parse().unwrap_or_default(),
                    record.get(3).unwrap_or("").trim().to_string(),
                    record.get(4).unwrap_or("").trim().to_string(),
                    record.get(5).unwrap_or("0").trim().parse().unwrap_or(0.0),
                    record.get(6).unwrap_or("").trim().to_string(),
                    0,
                )
            }
            7 => {
                // v2 - macOS SwiftUI
                let date_format = "%Y-%m-%d %H:%M:%S";
                FurTask::new_with_last_updated(
                    record.get(0).unwrap_or("").to_string(),
                    Local
                        .from_local_datetime(
                            &NaiveDateTime::parse_from_str(
                                record.get(4).unwrap_or(""),
                                date_format,
                            )
                            .unwrap_or_default(),
                        )
                        .single()
                        .unwrap_or_default(),
                    Local
                        .from_local_datetime(
                            &NaiveDateTime::parse_from_str(
                                record.get(5).unwrap_or(""),
                                date_format,
                            )
                            .unwrap_or_default(),
                        )
                        .single()
                        .unwrap_or_default(),
                    record
                        .get(2)
                        .unwrap_or("")
                        .split('#')
                        .map(|t| t.trim().to_lowercase())
                        .filter(|t| !t.is_empty())
                        .sorted()
                        .unique()
                        .collect::<Vec<String>>()
                        .join(" #"),
                    record.get(1).unwrap_or("").trim().to_string(),
                    record.get(3).unwrap_or("0").trim().parse().unwrap_or(0.0),
                    String::new(),
                    0,
                )
            }
            6 => FurTask::new_with_last_updated(
                // v1 - GTK
                record.get(1).unwrap_or("").to_string(),
                record.get(2).unwrap_or("").parse().unwrap_or_default(),
                record.get(3).unwrap_or("").parse().unwrap_or_default(),
                record
                    .get(4)
                    .unwrap_or("")
                    .split('#')
                    .map(|t| t.trim().to_lowercase())
                    .filter(|t| !t.is_empty())
                    .sorted()
                    .unique()
                    .collect::<Vec<String>>()
                    .join(" #"),
                String::new(),
                0.0,
                String::new(),
                0,
            ),

            _ => return Err(localization.get_message("invalid-csv", None).into()),
        };

        if let Ok(exists) = db_task_exists(&task) {
            if !exists {
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

pub fn import_csv_to_database(file: &mut File, localization: &Localization) {
    // Seek back to the start of the file after verification
    if let Err(e) = file.seek(std::io::SeekFrom::Start(0)) {
        eprintln!("Failed to seek to start of file: {}", e);
        return;
    }

    match read_csv(file, localization) {
        Ok(tasks_to_import) => {
            if let Err(e) = db_insert_tasks(&tasks_to_import) {
                eprintln!("Failed to import tasks: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to read the CSV file: {}", e),
    }
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    alert: Container<'a, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            center(opaque(row![horizontal_space(), alert, horizontal_space()])).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            })
        )
    ]
    .into()
}

fn format_iced_time_as_hm(time: iced_aw::time_picker::Time) -> String {
    let naive_time = NaiveTime::from(time);
    naive_time.format("%H:%M").to_string()
}

fn reset_fur_user(user: &mut Option<FurUser>) {
    *user = None;
    match db_delete_all_credentials() {
        Ok(_) => {}
        Err(e) => eprintln!("Error deleting user credentials: {}", e),
    }
}

fn set_positive_temp_notice(
    message_holder: &mut Result<String, Box<dyn std::error::Error>>,
    message: String,
) -> Task<Message> {
    *message_holder = Ok(message);
    Task::perform(
        async {
            tokio::time::sleep(std::time::Duration::from_secs(SETTINGS_MESSAGE_DURATION)).await;
        },
        |_| Message::ClearLoginMessage,
    )
}

fn set_negative_temp_notice(
    message_holder: &mut Result<String, Box<dyn std::error::Error>>,
    message: String,
) -> Task<Message> {
    *message_holder = Err(message.into());
    Task::perform(
        async {
            tokio::time::sleep(std::time::Duration::from_secs(SETTINGS_MESSAGE_DURATION)).await;
        },
        |_| Message::ClearLoginMessage,
    )
}

pub fn sync_after_change(user: &Option<FurUser>) -> Task<Message> {
    if user.is_some() {
        Task::perform(
            async {
                // Small delay to allow any pending DB operations to complete
                time::sleep(Duration::from_secs(1)).await;
            },
            |_| Message::SyncWithServer,
        )
    } else {
        Task::none()
    }
}
