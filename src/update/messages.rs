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

use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    path::Path,
};

use crate::{
    app::{Furtherance, write_furtasks_to_csv},
    autosave::write_autosave,
    constants::{ALLOWED_DB_EXTENSIONS, OFFICIAL_SERVER},
    database::*,
    helpers::{
        color_utils::{RandomColor, ToHex},
        idle, task_actions,
    },
    models::{
        fur_idle::FurIdle,
        fur_shortcut::{EncryptedShortcut, FurShortcut},
        fur_task::{EncryptedTask, FurTask},
        fur_task_group::FurTaskGroup,
        fur_todo::{EncryptedTodo, FurTodo, TodoToAdd, TodoToEdit},
        fur_user::FurUserFields,
        group_to_edit::GroupToEdit,
        shortcut_to_add::ShortcutToAdd,
        shortcut_to_edit::ShortcutToEdit,
        task_to_add::TaskToAdd,
        task_to_edit::TaskToEdit,
    },
    server::{
        encryption::{self, decrypt_encryption_key, encrypt_encryption_key},
        login::{ApiError, LoginResponse, login},
        logout,
        sync::{SyncResponse, sync_with_server},
    },
    update::msg_helper_functions::{
        chain_tasks, combine_chosen_date_with_time, combine_chosen_time_with_date,
        convert_iced_time_to_chrono_local, get_stopped_timer_text, get_timer_duration,
        get_timer_text, has_max_two_decimals, import_csv_to_database, reset_fur_user, reset_timer,
        set_negative_temp_notice, set_positive_temp_notice, show_notification, start_timer,
        stop_timer, sync_after_change, update_task_history, update_todo_list, verify_csv,
    },
    view_enums::*,
};
use chrono::{Local, NaiveDate, TimeDelta, TimeZone, offset::LocalResult};
use fluent::FluentValue;
use iced::{
    Color, Task, font,
    widget::{self},
};
use iced_aw::{date_picker, time_picker};
use itertools::Itertools;
use palette::Srgb;
use rfd::FileDialog;
use webbrowser;

#[derive(Debug, Clone)]
pub enum Message {
    AddNewShortcutPressed,
    AddNewTaskPressed,
    AddNewTodoPressed,
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
    CancelTodoEdit,
    CancelTodoEditDate,
    ChartTaskPropertyKeySelected(FurTaskProperty),
    ChartTaskPropertyValueSelected(String),
    ChooseCurrentTaskStartTime,
    ChooseEndDate,
    ChooseShortcutColor,
    ChooseStartDate,
    ChooseTaskEditDateTime(EditTaskProperty),
    ChooseTodoEditDate,
    ClearLoginMessage,
    CreateShortcutFromTaskGroup(FurTaskGroup),
    DeleteEverything,
    DateRangeSelected(FurDateRange),
    DeleteShortcut,
    DeleteShortcutFromContext(String),
    DeleteTasks,
    DeleteTasksFromContext(Vec<String>),
    DeleteTodo,
    DeleteTodoPressed(String),
    Done,
    EditGroup(FurTaskGroup),
    EditShortcutPressed(FurShortcut),
    EditShortcutTextChanged(String, EditTaskProperty),
    EditTask(FurTask),
    EditTaskTextChanged(String, EditTaskProperty),
    EditTodoTextChanged(String, EditTodoProperty),
    EditTodo(FurTodo),
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
    RepeatTodoToday(FurTodo),
    ReportTabSelected(TabId),
    SaveGroupEdit,
    SaveShortcut,
    SaveTaskEdit,
    SaveTodoEdit,
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
    SettingsShowDailyTimeTotalToggled(bool),
    SettingsShowEarningsToggled(bool),
    SettingsShowSecondsToggled(bool),
    SettingsShowTaskProjectToggled(bool),
    SettingsShowTaskTagsToggled(bool),
    SettingsShowTodoProjectToggled(bool),
    SettingsShowTodoRateToggled(bool),
    SettingsShowTodoTagsToggled(bool),
    SettingsTabSelected(TabId),
    ShortcutPressed(String),
    ShowAlert(FurAlert),
    StartStopPressed,
    StartTimerWithTask(String),
    StopwatchTick,
    SubmitCurrentTaskStartTime(time_picker::Time),
    SubmitEndDate(date_picker::Date),
    SubmitShortcutColor(Color),
    SubmitStartDate(date_picker::Date),
    SubmitTaskEditDate(date_picker::Date, EditTaskProperty),
    SubmitTaskEditTime(time_picker::Time, EditTaskProperty),
    SubmitTodoEditDate(date_picker::Date),
    SyncWithServer,
    SyncComplete((Result<SyncResponse, ApiError>, usize)),
    TabPressed { shift: bool },
    TaskInputChanged(String),
    ToggleGroupEditor,
    ToggleTodoCompletePressed(String),
    UpdateTaskHistory(BTreeMap<NaiveDate, Vec<FurTaskGroup>>),
    UpdateTodaysTodos(Vec<FurTodo>),
    UpdateTodoList(BTreeMap<NaiveDate, Vec<FurTodo>>),
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
            Message::AddNewTodoPressed => {
                self.todo_to_add = Some(TodoToAdd::new());
                self.inspector_view = Some(FurInspectorView::AddNewTodo);
            }
            Message::AddTaskToGroup(group_to_edit) => {
                self.task_to_add = Some(TaskToAdd::new_from(&group_to_edit));
                self.inspector_view = Some(FurInspectorView::AddTaskToGroup);
            }
            Message::AlertClose => {
                self.delete_tasks_from_context = None;
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
            Message::CancelTodoEdit => {
                self.todo_to_edit = None;
                self.todo_to_add = None;
                self.inspector_view = None;
            }
            Message::CancelTodoEditDate => {
                if let Some(todo_to_edit) = self.todo_to_edit.as_mut() {
                    todo_to_edit.show_date_picker = false;
                } else if let Some(todo_to_add) = self.todo_to_add.as_mut() {
                    todo_to_add.show_date_picker = false;
                }
            }
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
            Message::ChooseTodoEditDate => {
                if let Some(todo_to_edit) = self.todo_to_edit.as_mut() {
                    todo_to_edit.show_date_picker = true;
                } else if let Some(todo_to_add) = self.todo_to_add.as_mut() {
                    todo_to_add.show_date_picker = true;
                }
            }
            Message::ClearLoginMessage => {
                if self
                    .login_message
                    .iter()
                    .any(|message| message != &self.localization.get_message("syncing", None))
                {
                    self.login_message = Ok(String::new());
                }
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
                    match db_retrieve_existing_shortcuts() {
                        Ok(shortcuts) => self.shortcuts = shortcuts,
                        Err(e) => {
                            eprintln!("Failed to retrieve shortcuts from database: {}", e)
                        }
                    };
                    let mut tasks = vec![];
                    tasks.push(update_task_history(self.fur_settings.days_to_show));
                    tasks.push(update_todo_list());
                    tasks.push(sync_after_change(&self.fur_user));
                    return chain_tasks(tasks);
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
                if let Some(tasks_to_delete) = &self.delete_tasks_from_context {
                    if let Err(e) = db_delete_tasks_by_ids(tasks_to_delete) {
                        eprintln!("Failed to delete tasks: {}", e);
                    }
                    self.delete_tasks_from_context = None;
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    return update_task_history(self.fur_settings.days_to_show);
                } else if let Some(task_to_edit) = &self.task_to_edit {
                    self.inspector_view = None;
                    if let Err(e) = db_delete_tasks_by_ids(&[task_to_edit.uid.clone()]) {
                        eprintln!("Failed to delete task: {}", e);
                    }
                    self.task_to_edit = None;
                    self.displayed_alert = None;
                    return update_task_history(self.fur_settings.days_to_show);
                } else if let Some(group_to_edit) = &self.group_to_edit {
                    self.inspector_view = None;
                    if let Err(e) = db_delete_tasks_by_ids(&group_to_edit.all_task_ids()) {
                        eprintln!("Failed to delete tasks: {}", e);
                    }
                    self.group_to_edit = None;
                    self.displayed_alert = None;
                    return update_task_history(self.fur_settings.days_to_show);
                }
            }
            Message::DeleteTasksFromContext(task_group_ids) => {
                let number_of_tasks = task_group_ids.len();
                self.delete_tasks_from_context = Some(task_group_ids);
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
            Message::DeleteTodo => {
                if let Some(todo_to_delete) = &self.delete_todo_uid {
                    if let Err(e) = db_delete_todo_by_id(todo_to_delete) {
                        eprintln!("Failed to delete todo: {}", e);
                    }
                    self.delete_todo_uid = None;
                    self.inspector_view = None;
                    self.todo_to_edit = None;
                    self.displayed_alert = None;
                    return update_todo_list();
                } else if let Some(todo_to_edit) = &self.todo_to_edit {
                    self.inspector_view = None;
                    if let Err(e) = db_delete_todo_by_id(&todo_to_edit.uid.clone()) {
                        eprintln!("Failed to delete todo: {}", e);
                    }
                    self.todo_to_edit = None;
                    self.displayed_alert = None;
                    return update_todo_list();
                }
            }
            Message::DeleteTodoPressed(id) => {
                self.delete_todo_uid = Some(id);
                let delete_confirmation = self.fur_settings.show_delete_confirmation;
                return Task::perform(
                    async move {
                        if delete_confirmation {
                            Message::ShowAlert(FurAlert::DeleteTodoConfirmation)
                        } else {
                            Message::DeleteTodo
                        }
                    },
                    |msg| msg,
                );
            }
            Message::Done => {}
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
            Message::EditTodoTextChanged(new_value, property) => match self.inspector_view {
                Some(FurInspectorView::AddNewTodo) => {
                    if let Some(todo_to_add) = self.todo_to_add.as_mut() {
                        match property {
                            EditTodoProperty::Task => {
                                if new_value.contains('#')
                                    || new_value.contains('@')
                                    || new_value.contains('$')
                                {
                                    todo_to_add.invalid_input_error_message =
                                        self.localization.get_message("name-cannot-contain", None);
                                } else {
                                    todo_to_add.name = new_value;
                                    todo_to_add.invalid_input_error_message = String::new();
                                }
                            }
                            EditTodoProperty::Project => {
                                if new_value.contains('#')
                                    || new_value.contains('@')
                                    || new_value.contains('$')
                                {
                                    todo_to_add.input_error(
                                        self.localization
                                            .get_message("project-cannot-contain", None),
                                    );
                                } else {
                                    todo_to_add.project = new_value;
                                }
                            }
                            EditTodoProperty::Tags => {
                                if new_value.contains('@') || new_value.contains('$') {
                                    todo_to_add.input_error(
                                        self.localization.get_message("tags-cannot-contain", None),
                                    );
                                } else if !new_value.is_empty()
                                    && new_value.chars().next() != Some('#')
                                {
                                    todo_to_add.input_error(
                                        self.localization.get_message("tags-must-start", None),
                                    );
                                } else {
                                    todo_to_add.tags = new_value;
                                    todo_to_add.input_error(String::new());
                                }
                            }
                            EditTodoProperty::Rate => {
                                let new_value_parsed = new_value.parse::<f32>();
                                if new_value.is_empty() {
                                    todo_to_add.rate = String::new();
                                } else if new_value.contains('$') {
                                    todo_to_add.input_error(
                                        self.localization.get_message("no-symbol-in-rate", None),
                                    );
                                } else if new_value_parsed.is_ok()
                                    && has_max_two_decimals(&new_value)
                                    && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                {
                                    todo_to_add.rate = new_value;
                                    todo_to_add.input_error(String::new());
                                } else {
                                    todo_to_add.input_error(
                                        self.localization.get_message("rate-invalid", None),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Some(FurInspectorView::EditTodo) => {
                    if let Some(todo_to_edit) = self.todo_to_edit.as_mut() {
                        match property {
                            EditTodoProperty::Task => {
                                if new_value.contains('#')
                                    || new_value.contains('@')
                                    || new_value.contains('$')
                                {
                                    todo_to_edit.input_error(
                                        self.localization.get_message("name-cannot-contain", None),
                                    );
                                } else {
                                    todo_to_edit.new_name = new_value;
                                    todo_to_edit.input_error(String::new());
                                }
                            }
                            EditTodoProperty::Project => {
                                if new_value.contains('#')
                                    || new_value.contains('@')
                                    || new_value.contains('$')
                                {
                                    todo_to_edit.input_error(
                                        self.localization
                                            .get_message("project-cannot-contain", None),
                                    );
                                } else {
                                    todo_to_edit.new_project = new_value;
                                }
                            }
                            EditTodoProperty::Tags => {
                                if new_value.contains('@') || new_value.contains('$') {
                                    todo_to_edit.input_error(
                                        self.localization.get_message("tags-cannot-contain", None),
                                    );
                                } else if !new_value.is_empty()
                                    && new_value.chars().next() != Some('#')
                                {
                                    todo_to_edit.input_error(
                                        self.localization.get_message("tags-must-start", None),
                                    );
                                } else {
                                    todo_to_edit.new_tags = new_value;
                                    todo_to_edit.input_error(String::new());
                                }
                            }
                            EditTodoProperty::Rate => {
                                let new_value_parsed = new_value.parse::<f32>();
                                if new_value.is_empty() {
                                    todo_to_edit.new_rate = String::new();
                                } else if new_value.contains('$') {
                                    todo_to_edit.input_error(
                                        self.localization.get_message("no-symbol-in-rate", None),
                                    );
                                } else if new_value_parsed.is_ok()
                                    && has_max_two_decimals(&new_value)
                                    && new_value_parsed.unwrap_or(f32::MAX) < f32::MAX
                                {
                                    todo_to_edit.new_rate = new_value;
                                    todo_to_edit.input_error(String::new());
                                } else {
                                    todo_to_edit.input_error(
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
            Message::EditTodo(todo_to_edit) => {
                self.todo_to_edit = Some(TodoToEdit::new_from(&todo_to_edit));
                self.inspector_view = Some(FurInspectorView::EditTodo);
            }
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
                let mut tasks = vec![];
                tasks.push(update_task_history(self.fur_settings.days_to_show));
                tasks.push(sync_after_change(&self.fur_user));
                return chain_tasks(tasks);
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

                                return update_task_history(self.fur_settings.days_to_show);
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
                self.displayed_alert = None;
                match db_import_old_mac_db() {
                    Ok(_) => {
                        // Always do a full sync after import
                        if let Err(e) = self.fur_settings.change_needs_full_sync(&true) {
                            eprintln!("Error changing needs_full_sync: {}", e);
                        };

                        return update_task_history(self.fur_settings.days_to_show);
                    }
                    Err(e) => {
                        eprintln!("Error importing existing Core Data database: {e}")
                    }
                }
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
                let mut tasks = vec![];
                tasks.push(update_task_history(self.fur_settings.days_to_show));
                tasks.push(update_todo_list());
                return chain_tasks(tasks);
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
                let mut tasks = vec![];
                tasks.push(Task::perform(get_timer_duration(), |_| {
                    Message::StopwatchTick
                }));
                tasks.push(update_task_history(self.fur_settings.days_to_show));
                return chain_tasks(tasks);
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

                let mut tasks = vec![];
                tasks.push(Task::perform(get_timer_duration(), |_| {
                    Message::StopwatchTick
                }));
                tasks.push(update_task_history(self.fur_settings.days_to_show));
                tasks.push(sync_after_change(&self.fur_user));
                return chain_tasks(tasks);
            }
            Message::PomodoroStop => {
                self.pomodoro.snoozed = false;
                stop_timer(self, Local::now());
                self.displayed_alert = None;
                self.pomodoro.sessions = 0;
                let mut tasks = vec![];
                tasks.push(update_task_history(self.fur_settings.days_to_show));
                tasks.push(sync_after_change(&self.fur_user));
                return chain_tasks(tasks);
            }
            Message::PomodoroStopAfterBreak => {
                self.timer_is_running = false;
                self.pomodoro.on_break = false;
                self.pomodoro.snoozed = false;
                reset_timer(self);
                self.pomodoro.sessions = 0;
                self.displayed_alert = None;
                return update_task_history(self.fur_settings.days_to_show);
            }
            Message::RepeatLastTaskPressed(last_task_input) => {
                self.task_input = last_task_input;
                self.inspector_view = None;
                self.task_to_add = None;
                self.task_to_edit = None;
                self.current_view = FurView::Timer;
                return Task::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::RepeatTodoToday(todo_to_copy) => {
                match db_insert_todo(&FurTodo::new(
                    todo_to_copy.name,
                    todo_to_copy.project,
                    todo_to_copy.tags,
                    todo_to_copy.rate,
                    Local::now(),
                )) {
                    Ok(_) => {
                        let mut tasks = vec![];
                        tasks.push(update_todo_list());
                        tasks.push(sync_after_change(&self.fur_user));
                        return chain_tasks(tasks);
                    }
                    Err(e) => eprintln!("Error duplicating todo: {}", e),
                }
            }
            Message::ReportTabSelected(new_tab) => self.report.active_tab = new_tab,
            Message::SaveGroupEdit => {
                if let Some(group_to_edit) = &self.group_to_edit {
                    let _ = db_update_group_of_tasks(group_to_edit);
                    self.inspector_view = None;
                    self.group_to_edit = None;
                    let mut tasks = vec![];
                    tasks.push(update_task_history(self.fur_settings.days_to_show));
                    tasks.push(sync_after_change(&self.fur_user));
                    return chain_tasks(tasks);
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
                            let mut tasks = vec![];
                            tasks.push(update_task_history(self.fur_settings.days_to_show));
                            tasks.push(sync_after_change(&self.fur_user));
                            return chain_tasks(tasks);
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
                            let mut tasks = vec![];
                            tasks.push(update_task_history(self.fur_settings.days_to_show));
                            tasks.push(sync_after_change(&self.fur_user));
                            return chain_tasks(tasks);
                        }
                        Err(e) => eprintln!("Error adding task: {}", e),
                    }
                }
            }
            Message::SaveTodoEdit => {
                if let Some(todo_to_edit) = &self.todo_to_edit {
                    let tags_without_first_pound = todo_to_edit
                        .new_tags
                        .trim()
                        .split('#')
                        .filter_map(|tag| {
                            let trimmed = tag.trim();
                            if trimmed.is_empty() {
                                None
                            } else {
                                Some(trimmed.to_lowercase())
                            }
                        })
                        .sorted()
                        .collect::<Vec<String>>()
                        .join(" #");
                    match db_update_todo(&FurTodo {
                        name: todo_to_edit.new_name.trim().to_string(),
                        project: todo_to_edit.new_project.trim().to_string(),
                        tags: tags_without_first_pound,
                        rate: todo_to_edit.new_rate.trim().parse::<f32>().unwrap_or(0.0),
                        currency: String::new(),
                        date: todo_to_edit.new_date,
                        uid: todo_to_edit.uid.clone(),
                        is_completed: todo_to_edit.is_completed,
                        is_deleted: false,
                        last_updated: chrono::Utc::now().timestamp(),
                    }) {
                        Ok(_) => {
                            self.inspector_view = None;
                            self.todo_to_edit = None;
                            let mut tasks = vec![];
                            tasks.push(update_todo_list());
                            tasks.push(sync_after_change(&self.fur_user));
                            return chain_tasks(tasks);
                        }
                        Err(e) => eprintln!("Failed to update todo in database: {}", e),
                    }
                } else if let Some(todo_to_add) = &self.todo_to_add {
                    let tags_without_first_pound = todo_to_add
                        .tags
                        .trim()
                        .split('#')
                        .filter_map(|tag| {
                            let trimmed = tag.trim();
                            if trimmed.is_empty() {
                                None
                            } else {
                                Some(trimmed.to_lowercase())
                            }
                        })
                        .sorted()
                        .collect::<Vec<String>>()
                        .join(" #");
                    match db_insert_todo(&FurTodo::new(
                        todo_to_add.name.trim().to_string(),
                        todo_to_add.project.trim().to_string(),
                        tags_without_first_pound,
                        todo_to_add.rate.trim().parse::<f32>().unwrap_or(0.0),
                        todo_to_add.date,
                    )) {
                        Ok(_) => {
                            self.inspector_view = None;
                            self.todo_to_add = None;
                            let mut tasks = vec![];
                            tasks.push(update_todo_list());
                            tasks.push(sync_after_change(&self.fur_user));
                            return chain_tasks(tasks);
                        }
                        Err(e) => eprintln!("Error adding todo: {}", e),
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
                                        self.settings_database_message = Ok(match new_or_open {
                                            ChangeDB::Open => self
                                                .localization
                                                .get_message("database-loaded", None),
                                            ChangeDB::New => self
                                                .localization
                                                .get_message("database-created", None)
                                                .to_string(),
                                        });
                                        return update_task_history(self.fur_settings.days_to_show);
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
                        Ok(_) => {
                            return update_task_history(self.fur_settings.days_to_show);
                        }
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
                if let Err(e) = self
                    .fur_settings
                    .change_pomodoro_notification_alarm_sound(&new_value)
                {
                    eprintln!(
                        "Failed to change pomodoro_notification_alarm_sound in settings: {}",
                        e
                    );
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
                    show_notification(
                        NotificationType::Reminder,
                        &self.localization,
                        self.fur_settings.pomodoro_notification_alarm_sound,
                    );
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
            Message::SettingsShowDailyTimeTotalToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_daily_time_total(&new_value) {
                    eprintln!("Failed to change show_daily_time_total in settings: {}", e);
                }
            }
            Message::SettingsTabSelected(new_tab) => self.settings_active_tab = new_tab,
            Message::ShortcutPressed(shortcut_task_input) => {
                self.task_input = shortcut_task_input;
                self.inspector_view = None;
                self.shortcut_to_add = None;
                self.shortcut_to_edit = None;
                self.current_view = FurView::Timer;
                return Task::perform(async { Message::StartStopPressed }, |msg| msg);
            }
            Message::ShowAlert(alert_to_show) => self.displayed_alert = Some(alert_to_show),
            Message::SettingsShowEarningsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_task_earnings(&new_value) {
                    eprintln!("Failed to change show_earnings in settings: {}", e);
                }
            }
            Message::SettingsShowSecondsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_seconds(&new_value) {
                    eprintln!("Failed to change show_seconds in settings: {}", e);
                }
            }
            Message::SettingsShowTaskProjectToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_task_project(&new_value) {
                    eprintln!("Failed to change show_task_project in settings: {}", e);
                }
            }
            Message::SettingsShowTaskTagsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_task_tags(&new_value) {
                    eprintln!("Failed to change show_task_tags in settings: {}", e);
                }
            }
            Message::SettingsShowTodoProjectToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_todo_project(&new_value) {
                    eprintln!("Failed to change show_todo_project in settings: {}", e);
                }
            }
            Message::SettingsShowTodoRateToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_todo_rate(&new_value) {
                    eprintln!("Failed to change show_todo_rate in settings: {}", e);
                }
            }
            Message::SettingsShowTodoTagsToggled(new_value) => {
                if let Err(e) = self.fur_settings.change_show_todo_tags(&new_value) {
                    eprintln!("Failed to change show_todo_tags in settings: {}", e);
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
                        return update_task_history(self.fur_settings.days_to_show);
                    } else {
                        self.pomodoro.on_break = false;
                        self.pomodoro.snoozed = false;
                        self.pomodoro.sessions = 0;
                        stop_timer(self, Local::now());

                        let mut tasks = vec![];
                        tasks.push(update_task_history(self.fur_settings.days_to_show));
                        tasks.push(sync_after_change(&self.fur_user));
                        return chain_tasks(tasks);
                    }
                } else {
                    start_timer(self);
                    return Task::perform(get_timer_duration(), |_| Message::StopwatchTick);
                }
            }
            Message::StartTimerWithTask(task_input) => {
                self.task_input = task_input;
                self.inspector_view = None;
                self.task_to_add = None;
                self.task_to_edit = None;
                self.current_view = FurView::Timer;
                return Task::perform(async { Message::StartStopPressed }, |msg| msg);
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
                                show_notification(
                                    NotificationType::BreakOver,
                                    &self.localization,
                                    self.fur_settings.pomodoro_notification_alarm_sound,
                                );
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
                            show_notification(
                                NotificationType::Idle,
                                &self.localization,
                                self.fur_settings.pomodoro_notification_alarm_sound,
                            );
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
            Message::SubmitTodoEditDate(new_date) => {
                if let Some(todo_to_edit) = self.todo_to_edit.as_mut() {
                    if let LocalResult::Single(new_local_date_time) =
                        Local.with_ymd_and_hms(new_date.year, new_date.month, new_date.day, 0, 0, 0)
                    {
                        todo_to_edit.displayed_date = new_date;
                        todo_to_edit.new_date = new_local_date_time;
                        todo_to_edit.show_date_picker = false;
                    }
                } else if let Some(todo_to_add) = self.todo_to_add.as_mut() {
                    if let LocalResult::Single(new_local_date_time) =
                        Local.with_ymd_and_hms(new_date.year, new_date.month, new_date.day, 0, 0, 0)
                    {
                        todo_to_add.displayed_date = new_date;
                        todo_to_add.date = new_local_date_time;
                        todo_to_add.show_date_picker = false;
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
                        let new_todos: Vec<FurTodo>;

                        if needs_full_sync {
                            new_tasks =
                                db_retrieve_all_tasks(SortBy::StartTime, SortOrder::Ascending)
                                    .unwrap_or_default();
                            new_shortcuts = db_retrieve_all_shortcuts().unwrap_or_default();
                            new_todos = db_retrieve_all_todos().unwrap_or_default();
                        } else {
                            new_tasks =
                                db_retrieve_tasks_since_timestamp(last_sync).unwrap_or_default();
                            new_shortcuts = db_retrieve_shortcuts_since_timestamp(last_sync)
                                .unwrap_or_default();
                            new_todos =
                                db_retrieve_todos_since_timestamp(last_sync).unwrap_or_default();
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

                        let encrypted_todos: Vec<EncryptedTodo> = new_todos
                            .into_iter()
                            .filter_map(|todo| match encryption::encrypt(&todo, &encryption_key) {
                                Ok((encrypted_data, nonce)) => Some(EncryptedTodo {
                                    encrypted_data,
                                    nonce,
                                    uid: todo.uid,
                                    last_updated: todo.last_updated,
                                }),
                                Err(e) => {
                                    eprintln!("Failed to encrypt todo: {:?}", e);
                                    None
                                }
                            })
                            .collect();

                        let sync_count = encrypted_tasks.len()
                            + encrypted_shortcuts.len()
                            + encrypted_todos.len();

                        let sync_result = sync_with_server(
                            &user,
                            last_sync,
                            encrypted_tasks,
                            encrypted_shortcuts,
                            encrypted_todos,
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
                                                match db_update_task(&server_task) {
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Error updating task from server: {}",
                                                            e
                                                        );
                                                    }
                                                    _ => {
                                                        sync_count += 1;
                                                    }
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            // Task does not exist - insert it
                                            match db_insert_task(&server_task) {
                                                Err(e) => {
                                                    eprintln!(
                                                        "Error writing new task from server: {}",
                                                        e
                                                    );
                                                }
                                                _ => {
                                                    sync_count += 1;
                                                }
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
                                                match db_update_shortcut(&server_shortcut) {
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Error updating shortcut from server: {}",
                                                            e
                                                        );
                                                    }
                                                    _ => {
                                                        sync_count += 1;
                                                    }
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            // Shortcut does not exist - insert it
                                            match db_insert_shortcut(&server_shortcut) {
                                                Err(e) => {
                                                    eprintln!(
                                                        "Error writing new shortcut from server: {}",
                                                        e
                                                    );
                                                }
                                                _ => {
                                                    sync_count += 1;
                                                }
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

                        // Decrypt and process server todos
                        for encrypted_todo in response.todos {
                            match encryption::decrypt::<FurTodo>(
                                &encrypted_todo.encrypted_data,
                                &encrypted_todo.nonce,
                                &encryption_key,
                            ) {
                                Ok(server_todo) => {
                                    match db_retrieve_todo_by_id(&server_todo.uid) {
                                        Ok(Some(client_todo)) => {
                                            // Todo exists - update it if it changed
                                            if server_todo.last_updated > client_todo.last_updated {
                                                match db_update_todo(&server_todo) {
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Error updating todo from server: {}",
                                                            e
                                                        );
                                                    }
                                                    _ => {
                                                        sync_count += 1;
                                                    }
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            // Todo does not exist - insert it
                                            match db_insert_todo(&server_todo) {
                                                Err(e) => {
                                                    eprintln!(
                                                        "Error writing new todo from server: {}",
                                                        e
                                                    );
                                                }
                                                _ => {
                                                    sync_count += 1;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error checking for existing todo from server: {}",
                                                e
                                            )
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to decrypt todo: {:?}", e),
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
                            || !response.orphaned_todos.is_empty()
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

                            let orphaned_todos = if !response.orphaned_todos.is_empty() {
                                db_retrieve_orphaned_todos(response.orphaned_todos)
                                    .unwrap_or_default()
                            } else {
                                Vec::new()
                            };

                            if !orphaned_tasks.is_empty()
                                || !orphaned_shortcuts.is_empty()
                                || !orphaned_todos.is_empty()
                            {
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

                                        let encrypted_todos: Vec<EncryptedTodo> = orphaned_todos
                                            .into_iter()
                                            .filter_map(|todo| {
                                                match encryption::encrypt(&todo, &encryption_key) {
                                                    Ok((encrypted_data, nonce)) => {
                                                        Some(EncryptedTodo {
                                                            encrypted_data,
                                                            nonce,
                                                            uid: todo.uid,
                                                            last_updated: todo.last_updated,
                                                        })
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to encrypt todo: {:?}",
                                                            e
                                                        );
                                                        None
                                                    }
                                                }
                                            })
                                            .collect();

                                        sync_count += encrypted_tasks.len()
                                            + encrypted_shortcuts.len()
                                            + encrypted_todos.len();

                                        let sync_result = sync_with_server(
                                            &user,
                                            last_sync,
                                            encrypted_tasks,
                                            encrypted_shortcuts,
                                            encrypted_todos,
                                        )
                                        .await;

                                        (sync_result, sync_count)
                                    },
                                    Message::SyncComplete,
                                );
                            }
                        }

                        self.fur_settings.needs_full_sync = false;

                        let mut tasks = vec![];
                        tasks.push(update_task_history(self.fur_settings.days_to_show));
                        tasks.push(update_todo_list());
                        tasks.push(set_positive_temp_notice(
                            &mut self.login_message,
                            self.localization.get_message(
                                "sync-successful",
                                Some(&HashMap::from([("count", FluentValue::from(sync_count))])),
                            ),
                        ));
                        return chain_tasks(tasks);
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
                    return widget::operation::focus_previous();
                } else {
                    return widget::operation::focus_next();
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
            Message::ToggleTodoCompletePressed(uid) => match self
                .todos
                .values_mut()
                .flat_map(|vec| vec.iter_mut())
                .find(|todo| todo.uid == uid)
            {
                Some(todo) => {
                    todo.is_completed = !todo.is_completed;
                    match db_toggle_todo_completed(&uid) {
                        Ok(_) => return sync_after_change(&self.fur_user),
                        Err(e) => {
                            eprintln!(
                                "Failed to toggle is_completed on todo with uid {}: {}",
                                uid, e
                            );
                            match self
                                .todos
                                .values_mut()
                                .flat_map(|vec| vec.iter_mut())
                                .find(|todo| todo.uid == uid)
                            {
                                Some(todo_undo) => todo_undo.is_completed = !todo_undo.is_completed,
                                None => eprintln!(
                                    "Failed to undo toggle is_completed on todo with uid {}.",
                                    uid
                                ),
                            }
                        }
                    }
                }
                None => eprintln!("Failed to toggle is_completed on todo with uid {}.", uid),
            },
            Message::UpdateTaskHistory(new_history) => {
                self.task_history = new_history;

                let today = Local::now().date_naive();
                if let Some(todays_todos) = self.todos.get(&today) {
                    if let Some(todays_tasks) = self.task_history.get(&today) {
                        let todos_clone = todays_todos.clone();
                        let tasks_clone = todays_tasks.clone();

                        return Task::perform(
                            async move { task_actions::after_refresh(todos_clone, tasks_clone) },
                            |new_todos| Message::UpdateTodaysTodos(new_todos),
                        );
                    }
                };
            }
            Message::UpdateTodaysTodos(new_todos) => {
                let today = Local::now().date_naive();
                if let Some(todays_todos) = self.todos.get_mut(&today) {
                    *todays_todos = new_todos;
                }
            }
            Message::UpdateTodoList(new_list) => {
                self.todos = new_list;
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
                        let mut tasks: Vec<Task<Message>> = vec![];
                        tasks.push(set_positive_temp_notice(
                            &mut self.login_message,
                            self.localization.get_message("login-successful", None),
                        ));
                        tasks.push(sync_after_change(&self.fur_user));
                        return chain_tasks(tasks);
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
}
