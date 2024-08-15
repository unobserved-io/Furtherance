use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::fur_task::FurTask;

#[derive(Debug)]
pub struct FurTaskGroup {
    pub name: String,
    pub tags: String,
    pub project: String,
    pub rate: f32,
    pub total_time: i64,
    pub tasks: Vec<FurTask>,
}

impl FurTaskGroup {
    pub fn new_from(task: FurTask) -> Self {
        FurTaskGroup {
            name: task.name.clone(),
            tags: task.tags.clone(),
            project: task.project.clone(),
            rate: task.rate,
            total_time: (task.stop_time - task.start_time).num_seconds(),
            tasks: vec![task],
        }
    }

    pub fn add(&mut self, task: FurTask) {
        self.total_time += (task.stop_time - task.start_time).num_seconds();
        self.tasks.push(task);
    }

    pub fn is_equal_to(&self, task: &FurTask) -> bool {
        if self.name == task.name
            && self.tags == task.tags
            && self.project.to_lowercase() == task.project.to_lowercase()
            && self.rate == task.rate
        {
            true
        } else {
            false
        }
    }
}
// func sortTasks() {
// 	tasks.sort(by: { $0.startTime ?? Date.now > $1.startTime ?? Date.now })
// }
