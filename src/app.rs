// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
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

use std::collections::HashMap;

use crate::fur_task::FurTask;
use crate::style;
use crate::{database::*, fur_task_group::FurTaskGroup};
use chrono::{offset::LocalResult, DateTime, Datelike, Local, NaiveDate, NaiveTime};
use iced::widget::Row;
use iced::{
    alignment, font, keyboard,
    multi_window::Application,
    widget::{
        button, column, horizontal_space, pick_list, row, text, text_input, theme, vertical_space,
        Button, Column, Container, Scrollable,
    },
    window, Alignment, Command, Element, Font, Length, Renderer, Settings, Size, Theme,
};
use iced_aw::{
    core::icons::{bootstrap, BOOTSTRAP_FONT_BYTES, SF_UI_ROUNDED_BYTES},
    date_picker::{self, Date},
    modal,
    time_picker::{self, Period},
    Card, Modal, TimePicker, SF_UI_ROUNDED,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FurView {
    Shortcuts,
    Timer,
    History,
    Report,
    Settings,
}
impl Default for FurView {
    fn default() -> Self {
        FurView::Timer
    }
}

pub struct Furtherance {
    current_task_start_time: time_picker::Time,
    current_view: FurView,
    show_modal: bool,
    show_timer_start_picker: bool,
    task_history: HashMap<chrono::NaiveDate, Vec<FurTaskGroup>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),
    ModalClose,
    NavigateTo(FurView),
    CancelCurrentTaskStartTime,
    ChooseCurrentTaskStartTime,
    SubmitCurrentTaskStartTime(time_picker::Time),
}

impl Application for Furtherance {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        // Load or create database
        let _ = db_init();
        // Update old furtherance databases with new properties
        let _ = db_upgrade_old();

        let furtherance = Furtherance {
            current_task_start_time: time_picker::Time::now_hm(true),
            current_view: FurView::Timer,
            show_modal: false,
            show_timer_start_picker: false,
            task_history: get_task_history(),
        };

        (
            furtherance,
            font::load(BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
        )
    }

    fn title(&self, _window_id: window::Id) -> String {
        "Furtherance".to_owned()
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        match dark_light::detect() {
            dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            dark_light::Mode::Dark => Theme::Dark,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CancelCurrentTaskStartTime => {
                self.show_timer_start_picker = false;
                Command::none()
            }
            Message::ChooseCurrentTaskStartTime => {
                self.show_timer_start_picker = true;
                Command::none()
            }
            Message::FontLoaded(_) => Command::none(),
            Message::ModalClose => Command::none(),
            Message::NavigateTo(destination) => {
                self.current_view = destination;
                Command::none()
            }
            Message::SubmitCurrentTaskStartTime(new_time) => {
                match convert_iced_time_to_chrono_local(new_time) {
                    LocalResult::Single(local_time) => {
                        // TODO: Update start time for stopwatch to local_time
                        self.current_task_start_time = new_time;
                        self.show_timer_start_picker = false;
                    }
                    _ => {
                        self.show_timer_start_picker = false;
                        eprintln!("Error converting chosen time to local time.");
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self, _window_id: window::Id) -> Element<Message> {
        // MARK: SIDEBAR
        let sidebar = Container::new(
            column![
                nav_button("Shortcuts", FurView::Shortcuts),
                nav_button("Timer", FurView::Timer),
                nav_button("History", FurView::History),
                nav_button("Report", FurView::Report),
                vertical_space().height(Length::Fill),
                // TODO: if timer is running and nav is not timer, show timer
                nav_button("Settings", FurView::Settings)
            ]
            .spacing(12)
            .padding(20)
            .width(175)
            .align_items(Alignment::Start),
        )
        .style(style::gray_background);

        // MARK: Shortcuts
        let shortcuts_view = column![Scrollable::new(column![])];

        // MARK: TIMER
        let timer_view = column![
            row![
                button(bootstrap::icon_to_text(bootstrap::Bootstrap::ArrowRepeat))
                    .style(theme::Button::Text),
                horizontal_space().width(Length::Fill),
                text(format!("Recorded today: {}", "0:00"))
            ],
            vertical_space().height(Length::Fill),
            text("0:00:00").size(80),
            column![
                row![
                    text_input("", "").size(20),
                    button(row![
                        horizontal_space().width(Length::Fixed(5.0)),
                        bootstrap::icon_to_text(bootstrap::Bootstrap::PlayFill).size(20),
                        horizontal_space().width(Length::Fixed(5.0)),
                    ])
                ]
                .spacing(10),
                row![TimePicker::new(
                    self.show_timer_start_picker,
                    self.current_task_start_time,
                    Button::new(text(format!(
                        "Started at {}",
                        self.current_task_start_time.to_string()
                    )))
                    .on_press(Message::ChooseCurrentTaskStartTime),
                    Message::CancelCurrentTaskStartTime,
                    Message::SubmitCurrentTaskStartTime,
                )
                .use_24h(),]
                .align_items(Alignment::Center)
                .spacing(10),
            ]
            .align_items(Alignment::Center)
            .spacing(15),
            vertical_space().height(Length::Fill),
        ]
        .align_items(Alignment::Center)
        .padding(20);

        // MARK: HISTORY
        let mut all_history_rows: Column<'_, Message, Theme, Renderer> =
            Column::new().spacing(8).padding(20);
        for (date, task_groups) in &self.task_history {
            let total_time = task_groups
                .iter()
                .map(|group| group.total_time)
                .sum::<i64>();
            all_history_rows = all_history_rows.push(history_title_row(date, total_time));
            for task_group in task_groups {
                all_history_rows = all_history_rows.push(history_group_row(task_group))
            }
        }
        let history_view = column![Scrollable::new(all_history_rows)];

        // MARK: REPORT
        let report_view = column![Scrollable::new(column![])];

        // MARK: SETTINGS
        let settings_view = column![Scrollable::new(column![])];

        let content = row![
            sidebar,
            match self.current_view {
                FurView::Shortcuts => shortcuts_view,
                FurView::Timer => timer_view,
                FurView::History => history_view,
                FurView::Report => report_view,
                FurView::Settings => settings_view,
            },
        ];

        let overlay: Option<Card<'_, Message, Theme, Renderer>> = if self.show_modal {
            Some(
                Card::new(text("Title:"), text("Description")),
                // .foot(
                // )
            )
        } else {
            None
        };

        modal(content, overlay)
            .backdrop(Message::ModalClose)
            .on_esc(Message::ModalClose)
            .into()
    }
}

fn nav_button<'a>(text: &'a str, destination: FurView) -> Button<'a, Message> {
    button(text)
        .on_press(Message::NavigateTo(destination))
        .style(theme::Button::Text)
}

fn history_group_row<'a>(task_group: &FurTaskGroup) -> Container<'a, Message> {
    Container::new(row![])
}

fn history_title_row<'a>(date: &NaiveDate, total_time: i64) -> Row<'a, Message> {
    let h = total_time / 3600;
    let m = total_time % 3600 / 60;
    let s = total_time % 60;
    let total_time_str = format!("{:02}:{:02}:{:02}", h, m, s);
    // TODO: Change formatting if not showing seconds
    // if !show_seconds {
    //     total_time_str = format!("{:02}:{:02}", h, m);
    // }
    row![
        text(date.to_string()),
        horizontal_space().width(Length::Fill),
        text(total_time_str), // TODO: Change to formatted hms
    ]
}

fn get_task_history() -> HashMap<chrono::NaiveDate, Vec<FurTaskGroup>> {
    let mut grouped_tasks_by_date: HashMap<chrono::NaiveDate, Vec<FurTaskGroup>> = HashMap::new();

    //INFO : Change limit based on user limit or max limit. Also should limit by days not items.
    if let Ok(all_tasks) = db_retrieve_all(SortBy::StartTime, SortOrder::Descending) {
        let tasks_by_date = group_tasks_by_date(all_tasks);

        for (date, tasks) in tasks_by_date {
            let mut all_groups: Vec<FurTaskGroup> = vec![];
            for task in tasks {
                if let Some(matching_group) = all_groups.iter_mut().find(|x| x.is_equal_to(&task)) {
                    matching_group.add(task);
                } else {
                    all_groups.push(FurTaskGroup::new_from(task));
                }
            }
            grouped_tasks_by_date.insert(date, all_groups);
        }
    }
    grouped_tasks_by_date
}

fn group_tasks_by_date(tasks: Vec<FurTask>) -> HashMap<chrono::NaiveDate, Vec<FurTask>> {
    let mut grouped_tasks: HashMap<chrono::NaiveDate, Vec<FurTask>> = HashMap::new();

    for task in tasks {
        let date = task.start_time.date_naive(); // Extract the date part
        grouped_tasks
            .entry(date)
            .or_insert_with(Vec::new)
            .push(task);
    }

    grouped_tasks
}

fn convert_iced_time_to_chrono_local(iced_time: time_picker::Time) -> LocalResult<DateTime<Local>> {
    let (hour, minute, _) = match iced_time {
        time_picker::Time::Hm {
            hour,
            minute,
            period,
        } => (hour, minute, period),
        _ => (1, 1, Period::H24),
    };

    if let Some(time) = NaiveTime::from_hms_opt(hour, minute, 0) {
        Local::now().with_time(time)
    } else {
        LocalResult::None
    }
}
