use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

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
