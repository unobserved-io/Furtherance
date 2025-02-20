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
