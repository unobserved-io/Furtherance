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

use crate::{
    database::db_set_todo_completed,
    models::{fur_task_group::FurTaskGroup, fur_todo::FurTodo},
};

pub fn after_refresh(todays_todos: Vec<FurTodo>, todays_tasks: Vec<FurTaskGroup>) -> Vec<FurTodo> {
    let mut todos_new = todays_todos.clone();

    for todo in todos_new.iter_mut() {
        if let Some(_) = todays_tasks
            .iter()
            .find(|task_group| task_group.to_string() == todo.to_string())
        {
            match db_set_todo_completed(&todo.uid) {
                Ok(_) => todo.is_completed = true,
                Err(e) => eprintln!("Error while marking todo {} as completed: {}", todo.uid, e),
            }
        }
    }

    todos_new
}
