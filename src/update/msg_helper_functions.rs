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

use std::{fs::File, io::Seek, time::Duration};

use chrono::{
    DateTime, Datelike, Local, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike,
    offset::LocalResult,
};
use csv::{Reader, ReaderBuilder, StringRecord};
use iced::Task;
use iced_aw::{date_picker, time_picker};
use itertools::Itertools;
use notify_rust::{Notification, Timeout};
use regex::Regex;
use tokio::time;

use crate::{
    app::Furtherance,
    autosave::delete_autosave,
    constants::SETTINGS_MESSAGE_DURATION,
    database::{db_delete_all_credentials, db_insert_task, db_insert_tasks, db_task_exists},
    helpers::tasks,
    localization::Localization,
    models::{fur_idle::FurIdle, fur_task::FurTask, fur_user::FurUser},
    ui::todos,
    update::messages::Message,
    view_enums::NotificationType,
};

pub fn chain_tasks(commands: Vec<Task<Message>>) -> Task<Message> {
    Task::batch(commands)
}

pub fn update_task_history(days_to_show: i64) -> Task<Message> {
    Task::perform(
        async move { tasks::get_task_history(days_to_show) },
        Message::UpdateTaskHistory,
    )
}

pub fn update_todo_list() -> Task<Message> {
    Task::perform(
        async move { todos::get_all_todos() },
        Message::UpdateTodoList,
    )
}

pub fn set_positive_temp_notice(
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

pub fn set_negative_temp_notice(
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

pub fn has_max_two_decimals(input: &str) -> bool {
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

pub fn stop_timer(state: &mut Furtherance, stop_time: DateTime<Local>) {
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

pub fn start_timer(state: &mut Furtherance) {
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

pub async fn get_timer_duration() {
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

pub fn reset_timer(state: &mut Furtherance) {
    state.task_input = "".to_string();
    state.timer_text = get_timer_text(state, 0);
    state.idle = FurIdle::new();
}

fn convert_datetime_to_iced_time(dt: DateTime<Local>) -> time_picker::Time {
    time_picker::Time::from(dt.time())
}

pub fn get_timer_text(state: &Furtherance, seconds_elapsed: i64) -> String {
    if state.timer_is_running {
        get_running_timer_text(state, seconds_elapsed)
    } else {
        get_stopped_timer_text(state)
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

pub fn get_stopped_timer_text(state: &Furtherance) -> String {
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

pub fn seconds_to_formatted_duration(total_seconds: i64, show_seconds: bool) -> String {
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

pub fn combine_chosen_date_with_time(
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

pub fn combine_chosen_time_with_date(
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

pub fn show_notification(
    notification_type: NotificationType,
    localization: &Localization,
    notification_alarm_sound: bool,
) {
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
        .sound_name(if has_sound && notification_alarm_sound {
            "alarm-clock-elapsed"
        } else {
            ""
        })
        .timeout(Timeout::Milliseconds(6000))
        .show()
    {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to show notification: {e}"),
    }
}

pub fn convert_iced_time_to_chrono_local(
    iced_time: time_picker::Time,
) -> LocalResult<DateTime<Local>> {
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

pub fn reset_fur_user(user: &mut Option<FurUser>) {
    *user = None;
    match db_delete_all_credentials() {
        Ok(_) => {}
        Err(e) => eprintln!("Error deleting user credentials: {}", e),
    }
}
