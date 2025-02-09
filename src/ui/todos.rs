use std::collections::BTreeMap;

use chrono::{Datelike, Local, NaiveDate, TimeDelta};
use iced::{
    font,
    widget::{button, column, horizontal_space, row, text, Column, Container, Row},
    Alignment, Element, Length, Renderer, Theme,
};
use iced_aw::ContextMenu;
use iced_fonts::{bootstrap::icon_to_char, Bootstrap, BOOTSTRAP_FONT};

use crate::{
    app::Message,
    database,
    localization::Localization,
    models::{fur_settings::FurSettings, fur_todo::FurTodo},
    style,
};

pub fn get_all_todos() -> BTreeMap<chrono::NaiveDate, Vec<FurTodo>> {
    let future_limit = Local::now() + TimeDelta::days(3);
    let past_limit = Local::now() - TimeDelta::days(60);
    let mut todos_by_date: BTreeMap<chrono::NaiveDate, Vec<FurTodo>> = BTreeMap::new();

    match database::db_retrieve_todos_between_dates(
        past_limit.to_string(),
        future_limit.to_string(),
    ) {
        Ok(all_todos) => {
            todos_by_date = group_todos_by_date(all_todos);
        }
        Err(e) => {
            eprintln!("Error retrieving todos from database: {}", e);
        }
    }

    todos_by_date
}

fn group_todos_by_date(todos: Vec<FurTodo>) -> BTreeMap<chrono::NaiveDate, Vec<FurTodo>> {
    let mut grouped_todos: BTreeMap<chrono::NaiveDate, Vec<FurTodo>> = BTreeMap::new();

    for todo in todos {
        let date = todo.date.date_naive();
        grouped_todos
            .entry(date)
            .or_insert_with(Vec::new)
            .push(todo);
    }

    grouped_todos
}

pub fn todo_title_row<'a>(date: &NaiveDate, localization: &Localization) -> Row<'a, Message> {
    row![text(format_todo_date(date, localization)).font(font::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }),]
    .align_y(Alignment::Center)
}

fn format_todo_date(date: &NaiveDate, localization: &Localization) -> String {
    let today = Local::now().date_naive();
    let yesterday = today - TimeDelta::days(1);
    let tomorrow = today + TimeDelta::days(1);
    let current_year = today.year();

    if date == &today {
        localization.get_message("today", None)
    } else if date == &yesterday {
        localization.get_message("yesterday", None)
    } else if date == &tomorrow {
        localization.get_message("tomorrow", None)
    } else if date.year() == current_year {
        date.format("%b %d").to_string()
    } else {
        date.format("%b %d, %Y").to_string()
    }
}

pub fn todo_row<'a, 'loc>(
    todo: &'a FurTodo,
    timer_is_running: bool,
    settings: &'a FurSettings,
    localization: &'loc Localization,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'loc>, Message> {
    // TODO: Strikethrough task when completed
    let mut todo_properties: Column<'_, Message, Theme, Renderer> =
        column![text(todo.task.clone()).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),]
        .width(Length::FillPortion(6));
    if settings.show_project && !todo.project.is_empty() {
        todo_properties = todo_properties.push(text!("@{}", todo.project));
    }
    if settings.show_tags && !todo.tags.is_empty() {
        todo_properties = todo_properties.push(text!("#{}", todo.tags));
    }
    let todo_row: Row<'_, Message, Theme, Renderer> = row![
        // TODO: Checkbox,
        text(icon_to_char(if todo.is_completed {
            Bootstrap::CheckSquareFill
        } else {
            Bootstrap::CheckSquare
        }))
        .font(BOOTSTRAP_FONT),
        todo_properties,
        horizontal_space().width(Length::Fill),
        // TODO: Play button? If timer not running or crossed off. Maybe another symbol if this task is running with the timer
    ]
    .align_y(Alignment::Center)
    .spacing(5);

    let todo_clone = todo.clone();

    ContextMenu::new(
        todo_row,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button(text(localization.get_message("edit", None)))
                    .on_press(Message::EditTodo(todo_clone.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("delete", None)))
                    .on_press(Message::DeleteTodo(todo_clone.uid.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
            ])
            .max_width(150)
            .into()
        }),
    )
}
