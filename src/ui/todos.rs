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

use std::collections::BTreeMap;

use chrono::{Datelike, Local, NaiveDate, TimeDelta};
use iced::{
    font,
    widget::{button, column, horizontal_space, rich_text, row, span, text, Container, Row},
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
    let mut todo_extra_text: String = String::new();
    if settings.show_todo_project && !todo.project.is_empty() {
        todo_extra_text = format!("  @{}", todo.project);
    }
    if settings.show_todo_tags && !todo.tags.is_empty() {
        todo_extra_text = todo_extra_text + &format!("  #{}", todo.tags);
    }
    if settings.show_todo_rate && todo.rate > 0.0 {
        todo_extra_text = todo_extra_text + &format!("  ${}", todo.rate);
    }

    let todo_text: text::Rich<'_, Message, Theme, Renderer> = rich_text![
        span(todo.name.clone())
            .font(font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .strikethrough(todo.is_completed),
        span(todo_extra_text).strikethrough(todo.is_completed)
    ];

    let mut todo_row: Row<'_, Message, Theme, Renderer> = row![
        button(
            text(icon_to_char(if todo.is_completed {
                Bootstrap::CheckSquare
            } else {
                Bootstrap::Square
            }))
            .font(BOOTSTRAP_FONT)
        )
        .on_press(Message::ToggleTodoCompletePressed(todo.uid.clone()))
        .style(button::text),
        todo_text.width(Length::FillPortion(10)),
        horizontal_space().width(Length::FillPortion(1)),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    if !todo.is_completed && !timer_is_running {
        todo_row = todo_row.push(
            button(text(icon_to_char(Bootstrap::PlayFill)).font(BOOTSTRAP_FONT))
                .style(button::text)
                .on_press(Message::StartTimerWithTask(todo.to_string())),
        );
    }

    let todo_clone = todo.clone();

    ContextMenu::new(
        todo_row,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            let mut menu_items =
                column![
                    iced::widget::button(text(localization.get_message("edit", None)))
                        .on_press(Message::EditTodo(todo_clone.clone()))
                        .style(style::context_menu_button_style)
                        .width(Length::Fill),
                ];

            if todo_clone.date.date_naive() != Local::now().date_naive() {
                menu_items = menu_items.push(
                    iced::widget::button(text(localization.get_message("repeat-today", None)))
                        .on_press(Message::RepeatTodoToday(todo_clone.clone()))
                        .style(style::context_menu_button_style)
                        .width(Length::Fill),
                );
            }

            menu_items = menu_items.push(
                iced::widget::button(text(localization.get_message("delete", None)))
                    .on_press(Message::DeleteTodoPressed(todo_clone.uid.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
            );

            Container::new(menu_items).max_width(150).into()
        }),
    )
}
