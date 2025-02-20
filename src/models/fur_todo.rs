use chrono::{DateTime, Local, Utc};
use iced_aw::date_picker::Date;
use serde::{Deserialize, Serialize};

use super::fur_settings::FurSettings;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurTodo {
    pub task: String,
    pub project: String,
    pub tags: String,
    pub rate: f32,
    pub currency: String,
    pub date: DateTime<Local>,
    pub uid: String,
    pub is_completed: bool,
    pub is_deleted: bool,
    pub last_updated: i64,
}

impl FurTodo {
    pub fn new(
        task: String,
        project: String,
        tags: String,
        rate: f32,
        date: DateTime<Local>,
    ) -> Self {
        let uid = generate_todo_uid(&task, &date);

        FurTodo {
            task,
            project,
            tags,
            rate,
            currency: String::new(),
            date,
            uid,
            is_completed: false,
            is_deleted: false,
            last_updated: Utc::now().timestamp(),
        }
    }
}

impl ToString for FurTodo {
    fn to_string(&self) -> String {
        let mut task_string: String = self.task.to_string();

        if !self.project.is_empty() {
            task_string += &format!(" @{}", self.project);
        }
        if !self.tags.is_empty() {
            task_string += &format!(" #{}", self.tags);
        }
        if self.rate != 0.0 {
            task_string += &format!(" ${:.2}", self.rate);
        }

        task_string
    }
}

#[derive(Clone, Debug)]
pub struct TodoToAdd {
    pub task: String,
    pub project: String,
    pub tags: String,
    pub rate: f32,
    pub new_rate: String,
    pub date: DateTime<Local>,
    pub displayed_date: Date,
    pub show_date_picker: bool,
    pub invalid_input_error_message: String,
}

impl TodoToAdd {
    pub fn new() -> Self {
        let now = Local::now();
        TodoToAdd {
            task: String::new(),
            project: String::new(),
            tags: String::new(),
            rate: 0.0,
            new_rate: format!("{:.2}", 0.0),
            date: now,
            displayed_date: Date::from(now.date_naive()),
            show_date_picker: false,
            invalid_input_error_message: String::new(),
        }
    }

    pub fn input_error(&mut self, message: String) {
        self.invalid_input_error_message = message;
    }
}

pub struct TodoToEdit {
    pub task: String,
    pub new_task: String,
    pub date: DateTime<Local>,
    pub new_date: DateTime<Local>,
    pub displayed_date: Date,
    pub show_date_picker: bool,
    pub project: String,
    pub new_project: String,
    pub tags: String,
    pub new_tags: String,
    pub rate: f32,
    pub new_rate: String,
    pub uid: String,
    pub is_completed: bool,
    pub invalid_input_error_message: String,
}

impl TodoToEdit {
    pub fn new_from(todo: &FurTodo) -> Self {
        TodoToEdit {
            task: todo.task.clone(),
            new_task: todo.task.clone(),
            date: todo.date,
            new_date: todo.date,
            displayed_date: Date::from(todo.date.date_naive()),
            show_date_picker: false,
            project: todo.project.clone(),
            new_project: todo.project.clone(),
            tags: todo.tags.clone(),
            new_tags: if todo.tags.is_empty() {
                todo.tags.clone()
            } else {
                format!("#{}", todo.tags)
            },
            rate: todo.rate,
            new_rate: format!("{:.2}", todo.rate),
            uid: todo.uid.clone(),
            is_completed: todo.is_completed,
            invalid_input_error_message: String::new(),
        }
    }

    pub fn is_changed(&self) -> bool {
        if self.task != self.new_task.trim()
            || self.date != self.new_date
            || self.tags
                != self
                    .new_tags
                    .trim()
                    .strip_prefix('#')
                    .unwrap_or(&self.tags)
                    .trim()
            || self.project != self.new_project.trim()
            || self.rate != self.new_rate.trim().parse::<f32>().unwrap_or(0.0)
        {
            true
        } else {
            false
        }
    }

    pub fn input_error(&mut self, message: String) {
        self.invalid_input_error_message = message;
    }
}

pub fn generate_todo_uid(task: &str, date: &DateTime<Local>) -> String {
    let input = format!("{}{}", task, date.timestamp());
    blake3::hash(input.as_bytes()).to_hex().to_string()
}
