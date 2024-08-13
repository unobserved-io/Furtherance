use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurTask {
    pub id: u32,
    pub name: String,
    pub start_time: DateTime<Local>,
    pub stop_time: DateTime<Local>,
    pub tags: String,
    pub project: String,
    pub rate: f32,
}

impl ToString for FurTask {
    fn to_string(&self) -> String {
        let mut task_string: String = self.name.to_string();

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
