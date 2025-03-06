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

use std::time::Duration;

use iced::Task;
use tokio::time;

use crate::{
    app::Message, constants::SETTINGS_MESSAGE_DURATION, models::fur_user::FurUser, ui::todos,
};

use super::tasks;

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
