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

use core::f32;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use crate::{
    autosave::{autosave_exists, restore_autosave},
    constants::{
        FURTHERANCE_VERSION, INSPECTOR_ALIGNMENT, INSPECTOR_PADDING, INSPECTOR_SPACING,
        INSPECTOR_WIDTH, OFFICIAL_SERVER, SETTINGS_SPACING,
    },
    database::*,
    helpers::{
        color_utils::{FromHex, ToSrgb},
        midnight_subscription::MidnightSubscription,
        tasks,
    },
    localization::Localization,
    models::{
        fur_idle::FurIdle,
        fur_pomodoro::FurPomodoro,
        fur_report::FurReport,
        fur_settings::FurSettings,
        fur_shortcut::FurShortcut,
        fur_task_group::FurTaskGroup,
        fur_todo::{FurTodo, TodoToAdd, TodoToEdit},
        fur_user::{FurUser, FurUserFields},
        group_to_edit::GroupToEdit,
        shortcut_to_add::ShortcutToAdd,
        shortcut_to_edit::ShortcutToEdit,
        task_to_add::TaskToAdd,
        task_to_edit::TaskToEdit,
    },
    style::{self, FurTheme},
    ui::todos,
    update::{
        messages::Message,
        msg_helper_functions::{
            chain_tasks, get_timer_text, seconds_to_formatted_duration, split_task_input,
        },
    },
    view_enums::*,
};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, TimeDelta};
use csv::Writer;
use fluent::FluentValue;
use iced::{
    Alignment, Color, Element, Length, Padding, Renderer, Subscription, Task, Theme,
    advanced::subscription,
    alignment, font, keyboard,
    widget::{
        Button, Column, Container, Row, Scrollable, button, center, checkbox, column, container,
        opaque, pick_list, row, rule, space, stack, text, text_input, toggler,
    },
};
use iced_aw::{
    Card, ContextMenu, TabBarPosition, TabLabel, Tabs, TimePicker, color_picker, date_picker,
    number_input, time_picker,
};
use iced_fonts::bootstrap::{self, advanced_text};
use itertools::Itertools;
use palette::Srgb;
use palette::color_difference::Wcag21RelativeContrast;
use tokio::time;

#[cfg(target_os = "macos")]
use notify_rust::set_application;

pub struct Furtherance {
    pub current_view: FurView,
    pub delete_tasks_from_context: Option<Vec<String>>,
    pub delete_shortcut_from_context: Option<String>,
    pub delete_todo_uid: Option<String>,
    pub displayed_alert: Option<FurAlert>,
    pub displayed_task_start_time: time_picker::Time,
    pub fur_settings: FurSettings,
    pub fur_user: Option<FurUser>,
    pub fur_user_fields: FurUserFields,
    pub group_to_edit: Option<GroupToEdit>,
    pub idle: FurIdle,
    pub inspector_view: Option<FurInspectorView>,
    pub localization: Arc<Localization>,
    pub login_message: Result<String, Box<dyn std::error::Error>>,
    pub pomodoro: FurPomodoro,
    pub report: FurReport,
    pub settings_active_tab: TabId,
    pub settings_csv_message: Result<String, Box<dyn std::error::Error>>,
    pub settings_database_message: Result<String, Box<dyn std::error::Error>>,
    pub settings_more_message: Result<String, Box<dyn std::error::Error>>,
    pub settings_server_choice: Option<ServerChoices>,
    pub shortcuts: Vec<FurShortcut>,
    pub shortcut_to_add: Option<ShortcutToAdd>,
    pub shortcut_to_edit: Option<ShortcutToEdit>,
    pub show_timer_start_picker: bool,
    pub task_history: BTreeMap<NaiveDate, Vec<FurTaskGroup>>,
    pub task_input: String,
    pub task_to_add: Option<TaskToAdd>,
    pub task_to_edit: Option<TaskToEdit>,
    pub timer_is_running: bool,
    pub timer_start_time: DateTime<Local>,
    pub timer_text: String,
    pub todo_to_add: Option<TodoToAdd>,
    pub todo_to_edit: Option<TodoToEdit>,
    pub todos: BTreeMap<NaiveDate, Vec<FurTodo>>,
}

impl Furtherance {
    pub fn new() -> (Self, iced::Task<Message>) {
        // Load settings
        let mut settings = match FurSettings::new() {
            Ok(loaded_settings) => loaded_settings,
            Err(e) => {
                eprintln!("Error loading settings: {}", e);
                FurSettings::default()
            }
        };
        // Load or create database
        if let Err(e) = db_init() {
            if let Err(e) = settings.reset_to_default_db_location() {
                eprintln!("Error loading database. Can't load or save data: {}", e);
            }
            eprintln!(
                "Error loading database. Reverting to default location: {}",
                e
            );
        }

        // Load user credentials from database
        let saved_user = match db_retrieve_credentials() {
            Ok(optional_user) => optional_user,
            Err(e) => {
                eprintln!("Error retrieving user credentials from database: {}", e);
                None
            }
        };

        // Set application identifier for notifications
        #[cfg(target_os = "macos")]
        if let Err(e) = set_application("io.unobserved.furtherance") {
            eprintln!(
                "Failed to set application identifier for notifications: {}",
                e
            );
        }

        let mut furtherance = Furtherance {
            current_view: settings.default_view,
            delete_tasks_from_context: None,
            delete_shortcut_from_context: None,
            delete_todo_uid: None,
            displayed_alert: None,
            displayed_task_start_time: time_picker::Time::now_hm(true),
            fur_settings: settings,
            fur_user: saved_user.clone(),
            fur_user_fields: match &saved_user {
                Some(user) => FurUserFields {
                    email: user.email.clone(),
                    encryption_key: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
                    server: user.server.clone(),
                },
                None => FurUserFields::default(),
            },
            group_to_edit: None,
            idle: FurIdle::new(),
            localization: Arc::new(Localization::new()),
            login_message: Ok(String::new()),
            pomodoro: FurPomodoro::new(),
            inspector_view: None,
            report: FurReport::new(),
            settings_active_tab: TabId::General,
            settings_csv_message: Ok(String::new()),
            settings_database_message: Ok(String::new()),
            settings_more_message: Ok(String::new()),
            settings_server_choice: if saved_user
                .as_ref()
                .map_or(false, |user| user.server != OFFICIAL_SERVER)
            {
                Some(ServerChoices::Custom)
            } else {
                Some(ServerChoices::Official)
            },
            shortcuts: match db_retrieve_existing_shortcuts() {
                Ok(shortcuts) => shortcuts,
                Err(e) => {
                    eprintln!("Error reading shortcuts from database: {}", e);
                    vec![]
                }
            },
            shortcut_to_add: None,
            shortcut_to_edit: None,
            show_timer_start_picker: false,
            task_history: BTreeMap::<chrono::NaiveDate, Vec<FurTaskGroup>>::new(),
            task_input: "".to_string(),
            task_to_add: None,
            task_to_edit: None,
            timer_is_running: false,
            timer_start_time: Local::now(),
            timer_text: "0:00:00".to_string(),
            todo_to_add: None,
            todo_to_edit: None,
            todos: BTreeMap::<chrono::NaiveDate, Vec<FurTodo>>::new(),
        };

        furtherance.timer_text = get_timer_text(&furtherance, 0);

        if autosave_exists() {
            restore_autosave();
            if furtherance.displayed_alert == None {
                furtherance.displayed_alert = Some(FurAlert::AutosaveRestored);
            }
        }

        // Ask user to import old Furtherance database on first run
        if furtherance.fur_settings.first_run {
            #[cfg(target_os = "macos")]
            {
                furtherance.displayed_alert = db_check_for_existing_mac_db();
            }

            let _ = furtherance.fur_settings.change_first_run(false);
            let _ = furtherance.fur_settings.change_notify_of_sync(false);
        } else if furtherance.fur_settings.notify_of_sync {
            furtherance.displayed_alert = Some(FurAlert::NotifyOfSync)
        }

        furtherance.task_history = tasks::get_task_history(furtherance.fur_settings.days_to_show);

        furtherance.todos = todos::get_all_todos();

        let mut tasks: Vec<Task<Message>> = vec![];

        if furtherance.fur_user.is_some() {
            tasks.push(Task::perform(
                async {
                    // Small delay to allow startup operations to complete
                    // time::sleep(Duration::from_secs(1)).await;
                },
                |_| Message::SyncWithServer,
            ));
        }

        (furtherance, chain_tasks(tasks))
    }

    pub fn title(&self) -> String {
        "Furtherance".to_owned()
    }

    pub fn theme(&self) -> Theme {
        match dark_light::detect() {
            Ok(mode) => match mode {
                dark_light::Mode::Light | dark_light::Mode::Unspecified => {
                    FurTheme::Light.to_theme()
                }
                dark_light::Mode::Dark => FurTheme::Dark.to_theme(),
            },
            Err(_) => FurTheme::Light.to_theme(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let show_reminder_notification = if self.fur_settings.notify_reminder {
            Some(
                iced::time::every(time::Duration::from_secs(
                    (self.fur_settings.notify_reminder_interval * 60) as u64,
                ))
                .map(|_| Message::ShowReminderNotification),
            )
        } else {
            None
        };

        fn handle_hotkey(event: keyboard::Event) -> Option<Message> {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };

            match (key, modifiers) {
                (keyboard::Key::Named(keyboard::key::Named::Tab), _) => Some(Message::TabPressed {
                    shift: modifiers.shift(),
                }),
                _ => None,
            }
        }

        let timed_sync = if self.fur_user.is_some() {
            let sync_interval = 900; // 15 mins in secs
            Some(
                iced::time::every(Duration::from_secs(sync_interval))
                    .map(|_| Message::SyncWithServer),
            )
        } else {
            None
        };

        Subscription::batch([
            keyboard::listen().filter_map(handle_hotkey),
            subscription::from_recipe(MidnightSubscription),
            show_reminder_notification.unwrap_or(Subscription::none()),
            timed_sync.unwrap_or(Subscription::none()),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        // MARK: SIDEBAR
        let sidebar = Container::new(
            column![
                nav_button(
                    self.localization.get_message("shortcuts", None),
                    FurView::Shortcuts,
                    self.current_view == FurView::Shortcuts
                ),
                nav_button(
                    self.localization.get_message("timer", None),
                    FurView::Timer,
                    self.current_view == FurView::Timer
                ),
                nav_button(
                    self.localization.get_message("todo", None),
                    FurView::Todo,
                    self.current_view == FurView::Todo
                ),
                nav_button(
                    self.localization.get_message("report", None),
                    FurView::Report,
                    self.current_view == FurView::Report
                ),
                space::vertical().height(Length::Fill),
                if self.timer_is_running && self.current_view != FurView::Timer {
                    text(convert_timer_text_to_vertical_hms(
                        &self.timer_text,
                        &self.localization,
                    ))
                    .size(50)
                    .style(|theme| {
                        if self.pomodoro.on_break {
                            style::red_text(theme)
                        } else {
                            text::Style::default()
                        }
                    })
                } else {
                    text("")
                },
                nav_button(
                    self.localization.get_message("settings", None),
                    FurView::Settings,
                    self.current_view == FurView::Settings
                ),
            ]
            .spacing(12)
            .align_x(Alignment::Start),
        )
        .width(175)
        .padding(10)
        .clip(true)
        .style(style::gray_background);

        // MARK: Shortcuts
        let mut shortcuts_row = Row::new().spacing(20.0);
        for shortcut in &self.shortcuts {
            shortcuts_row = shortcuts_row.push(shortcut_button(
                shortcut,
                self.timer_is_running,
                &self.localization,
            ));
        }

        let new_shortcut_row = if self.inspector_view.is_none() {
            row![
                space::horizontal(),
                button(bootstrap::plus_lg())
                    .on_press(Message::AddNewShortcutPressed)
                    .style(button::text),
            ]
            .padding([10, 20])
        } else {
            row![button(" ").style(button::text)].padding([10, 20])
        };

        let shortcuts_view = column![
            new_shortcut_row,
            Scrollable::new(column![shortcuts_row.width(Length::Fill).wrap()].padding(20))
        ];

        // MARK: TIMER
        let mut all_history_rows: Column<'_, Message, Theme, Renderer> =
            Column::new().spacing(8).padding(Padding {
                top: 20.0,
                right: 20.0,
                bottom: 0.0,
                left: 20.0,
            });
        for (date, task_groups) in self.task_history.iter().rev() {
            let (total_time, total_earnings) = task_groups.iter().fold(
                (0i64, 0f32),
                |(accumulated_time, accumulated_earnings), group| {
                    let group_time = group.total_time;
                    let group_earnings = (group_time as f32 / 3600.0) * group.rate;

                    (
                        accumulated_time + group_time,
                        accumulated_earnings + group_earnings,
                    )
                },
            );
            all_history_rows = all_history_rows.push(history_title_row(
                date,
                total_time,
                total_earnings,
                &self.fur_settings,
                if self.timer_start_time.date_naive() == *date {
                    let (_, _, _, rate) = split_task_input(&self.task_input);
                    Some((self.timer_is_running, &self.timer_text, rate))
                } else {
                    None
                },
                &self.localization,
            ));
            for task_group in task_groups {
                all_history_rows = all_history_rows.push(history_group_row(
                    task_group,
                    self.timer_is_running,
                    &self.fur_settings,
                    &self.localization,
                ))
            }
        }

        let mut timer_view: Column<'_, Message> = column![].align_x(Alignment::Center).clip(true);
        timer_view = timer_view.push(if self.inspector_view.is_none() {
            row![
                space::horizontal(),
                button(bootstrap::plus_lg())
                    .on_press(Message::AddNewTaskPressed)
                    .style(button::text),
            ]
            .padding([10, 20])
        } else {
            row![button(" ").style(button::text)].padding([10, 20])
        });

        timer_view = timer_view.push(if self.task_history.is_empty() {
            Some(space::vertical())
        } else {
            None
        });

        timer_view = timer_view.push(text(&self.timer_text).size(80).style(|theme| {
            if self.pomodoro.on_break {
                style::red_text(theme)
            } else {
                text::Style::default()
            }
        }));
        timer_view = timer_view.push(
            column![
                row![
                    text_input(
                        &self
                            .localization
                            .get_message("task-input-placeholder", None),
                        &self.task_input
                    )
                    .on_input(Message::TaskInputChanged)
                    .on_submit(Message::EnterPressedInTaskInput)
                    .size(20),
                    button(row![
                        space::horizontal().width(Length::Fixed(5.0)),
                        if self.timer_is_running {
                            bootstrap::stop_fill().size(20)
                        } else {
                            bootstrap::play_fill().size(20)
                        },
                        space::horizontal().width(Length::Fixed(5.0)),
                    ])
                    .on_press_maybe(if self.task_input.trim().is_empty() {
                        None
                    } else {
                        Some(Message::StartStopPressed)
                    })
                    .style(style::primary_button_style),
                ]
                .spacing(10),
                if self.timer_is_running {
                    row![
                        TimePicker::new(
                            self.show_timer_start_picker,
                            self.displayed_task_start_time,
                            Button::new(text(self.localization.get_message(
                                "started-at",
                                Some(&HashMap::from([(
                                    "time",
                                    FluentValue::from(
                                        self.timer_start_time.format("%H:%M").to_string()
                                    )
                                )]))
                            )))
                            .on_press(Message::ChooseCurrentTaskStartTime)
                            .style(style::primary_button_style),
                            Message::CancelCurrentTaskStartTime,
                            Message::SubmitCurrentTaskStartTime,
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 10.0,
                        left: 0.0,
                    })
                } else {
                    row![]
                },
            ]
            .align_x(Alignment::Center)
            .spacing(15)
            .padding(Padding {
                top: 20.0,
                right: 20.0,
                bottom: 0.0,
                left: 20.0,
            }),
        );

        timer_view = timer_view.push(if self.task_history.is_empty() {
            Some(Scrollable::new(column![]).height(Length::Fill))
        } else {
            Some(Scrollable::new(all_history_rows).height(Length::Fill))
        });

        // MARK: TODOS
        let mut all_todo_rows: Column<'_, Message, Theme, Renderer> = Column::new()
            .spacing(8)
            .padding(Padding {
                top: 20.0,
                right: 20.0,
                bottom: 0.0,
                left: 20.0,
            })
            .width(Length::Fill);

        // First, check for today
        if let Some((date, todos)) = self
            .todos
            .iter()
            .find(|(date, _)| date == &&Local::now().date_naive())
        {
            all_todo_rows = all_todo_rows.push(todos::todo_title_row(&date, &self.localization));
            let mut today_column: Column<'_, Message, Theme, Renderer> = column![].spacing(8);
            for todo in todos.iter().sorted_by_key(|todo| todo.is_completed) {
                today_column = today_column.push(todos::todo_row(
                    todo,
                    self.timer_is_running,
                    &self.fur_settings,
                    &self.localization,
                ))
            }
            let today_container = Container::new(today_column)
                .padding([10, 0])
                .width(Length::Fill)
                .style(style::task_row);
            all_todo_rows = all_todo_rows.push(today_container);
        }
        // Next, check for tomorrow
        if let Some((date, todos)) = self
            .todos
            .iter()
            .find(|(date, _)| date == &&(Local::now().date_naive() + TimeDelta::days(1)))
        {
            all_todo_rows = all_todo_rows.push(todos::todo_title_row(&date, &self.localization));
            for todo in todos.iter().sorted_by_key(|todo| todo.is_completed) {
                all_todo_rows = all_todo_rows.push(todos::todo_row(
                    todo,
                    self.timer_is_running,
                    &self.fur_settings,
                    &self.localization,
                ))
            }
        }
        // Finally, show all other dates below
        for (date, todos) in self.todos.iter().rev() {
            if date != &Local::now().date_naive()
                && date != &(Local::now().date_naive() + TimeDelta::days(1))
            {
                all_todo_rows =
                    all_todo_rows.push(todos::todo_title_row(&date, &self.localization));
                for todo in todos.iter().sorted_by_key(|todo| todo.is_completed) {
                    all_todo_rows = all_todo_rows.push(todos::todo_row(
                        todo,
                        self.timer_is_running,
                        &self.fur_settings,
                        &self.localization,
                    ))
                }
            }
        }

        let mut todo_view: Column<'_, Message> = column![].align_x(Alignment::Center);
        // Add new todo button
        todo_view = todo_view.push(if self.inspector_view.is_none() {
            row![
                space::horizontal(),
                button(bootstrap::plus_lg())
                    .on_press(Message::AddNewTodoPressed)
                    .style(button::text),
            ]
            .padding([10, 20])
        } else {
            row![button(" ").style(button::text)].padding([10, 20])
        });

        todo_view = todo_view.push(Scrollable::new(all_todo_rows).height(Length::Fill));

        // MARK: REPORT
        let mut charts_column = Column::new().align_x(Alignment::Center);

        let mut timer_earnings_boxes_widgets: Vec<Element<'_, Message, Theme, Renderer>> =
            Vec::new();
        if self.fur_settings.show_chart_total_time_box && self.report.total_time > 0 {
            timer_earnings_boxes_widgets.push(
                column![
                    text(seconds_to_formatted_duration(self.report.total_time, true)).size(50),
                    text(self.localization.get_message("total-time", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }
        if self.fur_settings.show_chart_total_earnings_box && self.report.total_earned > 0.0 {
            timer_earnings_boxes_widgets.push(
                column![
                    text!("${:.2}", self.report.total_earned).size(50),
                    text(self.localization.get_message("earned", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }
        if !timer_earnings_boxes_widgets.is_empty() {
            // If both boxes are present, place a spacer between them
            if timer_earnings_boxes_widgets.len() == 2 {
                timer_earnings_boxes_widgets
                    .insert(1, space::horizontal().width(Length::Fill).into());
            }
            // Then place the bookend spacers
            timer_earnings_boxes_widgets.insert(0, space::horizontal().width(Length::Fill).into());
            timer_earnings_boxes_widgets.push(space::horizontal().width(Length::Fill).into());

            charts_column = charts_column.push(
                Row::with_children(timer_earnings_boxes_widgets).padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 10.0,
                    left: 0.0,
                }),
            );
        }

        if self.fur_settings.show_chart_time_recorded {
            charts_column = charts_column.push(self.report.time_recorded_chart.view());
        }
        if self.fur_settings.show_chart_earnings && self.report.total_earned > 0.0 {
            charts_column = charts_column.push(self.report.earnings_chart.view());
        }
        if self.fur_settings.show_chart_average_time {
            charts_column = charts_column.push(self.report.average_time_chart.view());
        }
        if self.fur_settings.show_chart_average_earnings && self.report.total_earned > 0.0 {
            charts_column = charts_column.push(self.report.average_earnings_chart.view());
        }

        // Breakdown by Selection Picker & Charts
        let mut selection_timer_earnings_boxes_widgets: Vec<Element<'_, Message, Theme, Renderer>> =
            Vec::new();
        if self.fur_settings.show_chart_total_time_box && self.report.selection_total_time > 0 {
            selection_timer_earnings_boxes_widgets.push(
                column![
                    text(seconds_to_formatted_duration(
                        self.report.selection_total_time,
                        true
                    ))
                    .size(50),
                    text(self.localization.get_message("total-time", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }
        if self.fur_settings.show_chart_total_earnings_box
            && self.report.selection_total_earned > 0.0
        {
            selection_timer_earnings_boxes_widgets.push(
                column![
                    text!("${:.2}", self.report.selection_total_earned).size(50),
                    text(self.localization.get_message("earned", None)),
                ]
                .align_x(Alignment::Center)
                .into(),
            );
        }

        let mut charts_breakdown_by_selection_column = Column::new().align_x(Alignment::Center);
        if !self.report.tasks_in_range.is_empty()
            && self.fur_settings.show_chart_breakdown_by_selection
        {
            charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                text(
                    self.localization
                        .get_message("breakdown-by-selection", None),
                )
                .size(40),
            );
            charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                row![
                    pick_list(
                        &FurTaskProperty::ALL[..],
                        self.report.picked_task_property_key,
                        Message::ChartTaskPropertyKeySelected,
                    )
                    .width(Length::Fill),
                    pick_list(
                        &self.report.task_property_value_keys[..],
                        self.report.picked_task_property_value.clone(),
                        Message::ChartTaskPropertyValueSelected,
                    )
                    .width(Length::Fill),
                ]
                .spacing(10)
                .width(Length::Fill),
            );
            charts_breakdown_by_selection_column =
                charts_breakdown_by_selection_column.push(rule::horizontal(20));

            if !selection_timer_earnings_boxes_widgets.is_empty() {
                // If both boxes are present, place a spacer between them
                if selection_timer_earnings_boxes_widgets.len() == 2 {
                    selection_timer_earnings_boxes_widgets
                        .insert(1, space::horizontal().width(Length::Fill).into());
                }
                // Then place the bookend spacers
                selection_timer_earnings_boxes_widgets
                    .insert(0, space::horizontal().width(Length::Fill).into());
                selection_timer_earnings_boxes_widgets
                    .push(space::horizontal().width(Length::Fill).into());

                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column.push(
                    Row::with_children(selection_timer_earnings_boxes_widgets).padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 10.0,
                        left: 0.0,
                    }),
                );
            }

            if self.fur_settings.show_chart_selection_time {
                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column
                    .push(self.report.selection_time_recorded_chart.view());
            }
            if self.fur_settings.show_chart_selection_earnings {
                charts_breakdown_by_selection_column = charts_breakdown_by_selection_column
                    .push(self.report.selection_earnings_recorded_chart.view());
            }
        };

        let charts_view = column![
            column![
                pick_list(
                    &FurDateRange::ALL[..],
                    self.report.picked_date_range,
                    Message::DateRangeSelected,
                )
                .width(Length::Fill),
                if self.report.picked_date_range == Some(FurDateRange::Range) {
                    row![
                        space::horizontal().width(Length::Fill),
                        date_picker(
                            self.report.show_start_date_picker,
                            self.report.picked_start_date,
                            button(text(self.report.picked_start_date.to_string()))
                                .on_press(Message::ChooseStartDate)
                                .style(style::primary_button_style),
                            Message::CancelStartDate,
                            Message::SubmitStartDate,
                        ),
                        column![
                            text("to")
                                .align_y(alignment::Vertical::Center)
                                .height(Length::Fill),
                        ]
                        .height(30),
                        date_picker(
                            self.report.show_end_date_picker,
                            self.report.picked_end_date,
                            button(text(self.report.picked_end_date.to_string()))
                                .on_press(Message::ChooseEndDate)
                                .style(style::primary_button_style),
                            Message::CancelEndDate,
                            Message::SubmitEndDate,
                        ),
                        space::horizontal().width(Length::Fill),
                    ]
                    .spacing(30)
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                } else {
                    row![]
                },
                space::vertical().height(Length::Fixed(20.0)),
                rule::horizontal(1),
            ]
            .padding(Padding {
                top: 20.0,
                right: 20.0,
                bottom: 10.0,
                left: 20.0,
            }),
            Scrollable::new(
                column![charts_column, charts_breakdown_by_selection_column]
                    .align_x(Alignment::Center)
                    .padding(Padding {
                        top: 0.0,
                        right: 20.0,
                        bottom: 20.0,
                        left: 20.0,
                    })
            ),
        ];

        // TODO: Change to tabbed report view once iced has lists
        // let report_view: Column<'_, Message, Theme, Renderer> =
        //     column![Tabs::new(Message::ReportTabSelected)
        //         .tab_icon_position(iced_aw::tabs::Position::Top)
        //         .push(
        //             TabId::Charts,
        //             TabLabel::IconText(
        //                 icon_to_char(Bootstrap::GraphUp),
        //                 "Charts".to_string()
        //             ),
        //             charts_view,
        //         )
        //         .push(
        //             TabId::List,
        //             TabLabel::IconText(
        //                 icon_to_char(Bootstrap::ListNested),
        //                 "List".to_string()
        //             ),
        //             Scrollable::new(column![].padding(10)),
        //         )
        //         .set_active_tab(&self.report.active_tab)
        //         .tab_bar_position(TabBarPosition::Top)];

        // MARK: SETTINGS
        let mut server_choice_col = column![pick_list(
            &ServerChoices::ALL[..],
            self.settings_server_choice,
            Message::SettingsServerChoiceSelected,
        ),]
        .padding([0, 15])
        .spacing(10);
        server_choice_col = server_choice_col.push(
            if self.settings_server_choice == Some(ServerChoices::Custom) {
                Some(
                    text_input("", &self.fur_user_fields.server)
                        .on_input(Message::UserServerChanged)
                        .on_submit(Message::EnterPressedInSyncFields),
                )
            } else {
                None
            },
        );

        let mut sync_server_col = column![
            row![
                text(self.localization.get_message("server", None)),
                server_choice_col,
            ]
            .align_y(Alignment::Center),
            row![
                text(self.localization.get_message("email", None)),
                column![
                    text_input("", &self.fur_user_fields.email)
                        .on_input(Message::UserEmailChanged)
                        .on_submit(Message::EnterPressedInSyncFields)
                ]
                .padding([0, 15])
            ]
            .align_y(Alignment::Center),
            row![
                text(self.localization.get_message("encryption-key", None)),
                column![
                    text_input("", &self.fur_user_fields.encryption_key)
                        .secure(true)
                        .on_input(Message::UserEncryptionKeyChanged)
                        .on_submit(Message::EnterPressedInSyncFields)
                ]
                .padding([0, 15])
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(10);
        let mut sync_button_row: Row<'_, Message> = row![
            button(text(self.localization.get_message(
                if self.fur_user.is_none() {
                    "log-in"
                } else {
                    "log-out"
                },
                None
            )))
            .on_press_maybe(if self.fur_user.is_none() {
                if !self.fur_user_fields.server.is_empty()
                    && !self.fur_user_fields.email.is_empty()
                    && !self.fur_user_fields.encryption_key.is_empty()
                {
                    Some(Message::UserLoginPressed)
                } else {
                    None
                }
            } else {
                Some(Message::UserLogoutPressed)
            })
            .style(if self.fur_user.is_none() {
                style::primary_button_style
            } else {
                button::secondary
            }),
        ]
        .spacing(10);
        sync_button_row = sync_button_row.push(if self.fur_user.is_some() {
            Some(
                button(text(self.localization.get_message("sync", None)))
                    .on_press_maybe(match self.fur_user {
                        Some(_) => {
                            if self.login_message.iter().any(|message| {
                                message != &self.localization.get_message("syncing", None)
                            }) {
                                Some(Message::SyncWithServer)
                            } else {
                                None
                            }
                        }
                        None => None,
                    })
                    .style(style::primary_button_style),
            )
        } else {
            Some(
                button(text(self.localization.get_message("sign-up", None)))
                    .on_press(Message::OpenUrl("https://furtherance.app/sync".to_string()))
                    .style(style::primary_button_style),
            )
        });
        sync_server_col = sync_server_col.push(sync_button_row);
        sync_server_col = sync_server_col.push(match &self.login_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let mut database_location_col = column![
            text(self.localization.get_message("database-location", None)),
            text_input(
                &self.fur_settings.database_url,
                &self.fur_settings.database_url,
            ),
            row![
                button(text(self.localization.get_message("create-new", None)))
                    .on_press(Message::SettingsChangeDatabaseLocationPressed(
                        ChangeDB::New
                    ))
                    .style(style::primary_button_style),
                button(text(self.localization.get_message("open-existing", None)))
                    .on_press(Message::SettingsChangeDatabaseLocationPressed(
                        ChangeDB::Open
                    ))
                    .style(style::primary_button_style),
                button(text(self.localization.get_message("backup-database", None)))
                    .on_press(Message::BackupDatabase)
                    .style(style::primary_button_style)
            ]
            .spacing(10)
            .wrap(),
        ]
        .spacing(10);
        database_location_col = database_location_col.push(match &self.settings_database_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let mut csv_col = column![
            row![
                button(text(self.localization.get_message("export-csv", None)))
                    .on_press(Message::ExportCsvPressed)
                    .style(style::primary_button_style),
                button(text(self.localization.get_message("import-csv", None)))
                    .on_press(Message::ImportCsvPressed)
                    .style(style::primary_button_style)
            ]
            .spacing(10),
        ]
        .spacing(10);
        csv_col = csv_col.push(match &self.settings_csv_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let mut backup_col = column![
            button(text(
                self.localization.get_message("delete-everything", None)
            ))
            .on_press(Message::ShowAlert(FurAlert::DeleteEverythingConfirmation))
            .style(button::danger)
        ]
        .spacing(10);
        backup_col = backup_col.push(match &self.settings_more_message {
            Ok(msg) => {
                if msg.is_empty() {
                    None
                } else {
                    Some(text(msg).style(style::green_text))
                }
            }
            Err(e) => Some(text!("{}", e).style(style::red_text)),
        });

        let settings_view: Column<'_, Message, Theme, Renderer> = column![
            Tabs::new(Message::SettingsTabSelected)
                .tab_icon_position(iced_aw::tabs::Position::Top)
                .push(
                    TabId::General,
                    TabLabel::IconText(
                        advanced_text::gear_fill().0.chars().next().unwrap_or(' '),
                        self.localization.get_message("general", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("interface", None)),
                            row![
                                text(self.localization.get_message("default-view", None)),
                                pick_list(
                                    &FurView::ALL[..],
                                    Some(self.fur_settings.default_view),
                                    Message::SettingsDefaultViewSelected,
                                ),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(
                                    self.localization
                                        .get_message("show-delete-confirmation", None)
                                ),
                                toggler(self.fur_settings.show_delete_confirmation)
                                    .on_toggle(Message::SettingsDeleteConfirmationToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("task-history", None)),
                            row![
                                text(self.localization.get_message("show-project", None)),
                                toggler(self.fur_settings.show_task_project)
                                    .on_toggle(Message::SettingsShowTaskProjectToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-tags", None)),
                                toggler(self.fur_settings.show_task_tags)
                                    .on_toggle(Message::SettingsShowTaskTagsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-earnings", None)),
                                toggler(self.fur_settings.show_task_earnings)
                                    .on_toggle(Message::SettingsShowEarningsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-seconds", None)),
                                toggler(self.fur_settings.show_seconds)
                                    .on_toggle(Message::SettingsShowSecondsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-daily-time-total", None)),
                                toggler(self.fur_settings.show_daily_time_total)
                                    .on_toggle(Message::SettingsShowDailyTimeTotalToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("todos", None)),
                            row![
                                text(self.localization.get_message("show-project", None)),
                                toggler(self.fur_settings.show_todo_project)
                                    .on_toggle(Message::SettingsShowTodoProjectToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-tags", None)),
                                toggler(self.fur_settings.show_todo_tags)
                                    .on_toggle(Message::SettingsShowTodoTagsToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("show-rate", None)),
                                toggler(self.fur_settings.show_todo_rate)
                                    .on_toggle(Message::SettingsShowTodoRateToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style),
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10)
                    ),
                )
                .push(
                    TabId::Advanced,
                    TabLabel::IconText(
                        advanced_text::gear_wide_connected()
                            .0
                            .chars()
                            .next()
                            .unwrap_or(' '),
                        self.localization.get_message("advanced", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("idle", None)),
                            row![
                                text(self.localization.get_message("idle-detection", None)),
                                toggler(self.fur_settings.notify_on_idle)
                                    .on_toggle(Message::SettingsIdleToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("minutes-until-idle", None)),
                                number_input(
                                    &self.fur_settings.chosen_idle_time,
                                    1..999,
                                    Message::SettingsIdleTimeChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("task-history", None)),
                            row![
                                column![
                                    text(self.localization.get_message("dynamic-total", None)),
                                    text(
                                        self.localization
                                            .get_message("dynamic-total-description", None)
                                    )
                                    .size(12),
                                ],
                                toggler(self.fur_settings.dynamic_total)
                                    .on_toggle(Message::SettingsDynamicTotalToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("days-to-show", None)),
                                number_input(
                                    &self.fur_settings.days_to_show,
                                    1..=365,
                                    Message::SettingsDaysToShowChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(
                                self.localization.get_message("reminder-notification", None)
                            ),
                            row![
                                column![
                                    text(
                                        self.localization
                                            .get_message("reminder-notifications", None)
                                    ),
                                    text(
                                        self.localization.get_message(
                                            "reminder-notifications-description",
                                            None
                                        )
                                    )
                                    .size(12),
                                ],
                                toggler(self.fur_settings.notify_reminder)
                                    .on_toggle(Message::SettingsRemindersToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("reminder-interval", None)),
                                number_input(
                                    &self.fur_settings.notify_reminder_interval,
                                    1..999,
                                    Message::SettingsReminderIntervalChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(format!("Furtherance version {}", FURTHERANCE_VERSION)).font(
                                    font::Font {
                                        style: iced::font::Style::Italic,
                                        ..Default::default()
                                    }
                                )
                            ]
                            .padding(Padding {
                                top: 40.0,
                                right: 0.0,
                                bottom: 0.0,
                                left: 0.0,
                            }),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10)
                    ),
                )
                .push(
                    TabId::Pomodoro,
                    TabLabel::IconText(
                        advanced_text::stopwatch_fill()
                            .0
                            .chars()
                            .next()
                            .unwrap_or(' '),
                        self.localization.get_message("pomodoro", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("pomodoro-timer", None)),
                            row![
                                text(self.localization.get_message("countdown-timer", None)),
                                toggler(self.fur_settings.pomodoro)
                                    .on_toggle_maybe(if self.timer_is_running {
                                        None
                                    } else {
                                        Some(Message::SettingsPomodoroToggled)
                                    })
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("timer-length", None)),
                                number_input(
                                    &self.fur_settings.pomodoro_length,
                                    1..999,
                                    Message::SettingsPomodoroLengthChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("break-length", None)),
                                number_input(
                                    &self.fur_settings.pomodoro_break_length,
                                    1..999,
                                    Message::SettingsPomodoroBreakLengthChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("snooze-length", None)),
                                number_input(
                                    &self.fur_settings.pomodoro_snooze_length,
                                    1..999,
                                    Message::SettingsPomodoroSnoozeLengthChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(
                                    self.localization
                                        .get_message("notification-alarm-sound", None)
                                ),
                                toggler(self.fur_settings.pomodoro_notification_alarm_sound)
                                    .on_toggle(
                                        Message::SettingsPomodoroNotificationAlarmSoundToggled
                                    )
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            settings_heading(self.localization.get_message("extended-break", None)),
                            row![
                                text(self.localization.get_message("extended-breaks", None)),
                                toggler(self.fur_settings.pomodoro_extended_breaks)
                                    .on_toggle(Message::SettingsPomodoroExtendedBreaksToggled)
                                    .width(Length::Shrink)
                                    .style(style::fur_toggler_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(
                                    self.localization
                                        .get_message("extended-break-interval", None)
                                ),
                                number_input(
                                    &self.fur_settings.pomodoro_extended_break_interval,
                                    1..999,
                                    Message::SettingsPomodoroExtendedBreakIntervalChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                            row![
                                text(self.localization.get_message("extended-break-length", None)),
                                number_input(
                                    &self.fur_settings.pomodoro_extended_break_length,
                                    1..999,
                                    Message::SettingsPomodoroExtendedBreakLengthChanged
                                )
                                .style(style::fur_number_input_style)
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                .push(
                    TabId::Report,
                    TabLabel::IconText(
                        advanced_text::graph_up().0.chars().next().unwrap_or(' '),
                        self.localization.get_message("report", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("toggle-charts", None)),
                            checkbox(self.fur_settings.show_chart_total_time_box)
                                .label(self.localization.get_message("total-time-box", None))
                                .on_toggle(Message::SettingsShowChartTotalTimeBoxToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_total_earnings_box)
                                .label(self.localization.get_message("total-earnings-box", None))
                                .on_toggle(Message::SettingsShowChartTotalEarningsBoxToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_time_recorded)
                                .label(self.localization.get_message("time-recorded", None))
                                .on_toggle(Message::SettingsShowChartTimeRecordedToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_earnings)
                                .label(self.localization.get_message("earnings", None))
                                .on_toggle(Message::SettingsShowChartEarningsToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_average_time)
                                .label(self.localization.get_message("average-time-per-task", None))
                                .on_toggle(Message::SettingsShowChartAverageTimeToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_average_earnings)
                                .label(
                                    self.localization
                                        .get_message("average-earnings-per-task", None)
                                )
                                .on_toggle(Message::SettingsShowChartAverageEarningsToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_breakdown_by_selection)
                                .label(
                                    self.localization
                                        .get_message("breakdown-by-selection-section", None)
                                )
                                .on_toggle(Message::SettingsShowChartBreakdownBySelectionToggled)
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_selection_time)
                                .label(
                                    self.localization
                                        .get_message("time-recorded-for-selection", None)
                                )
                                .on_toggle_maybe(
                                    if self.fur_settings.show_chart_breakdown_by_selection {
                                        Some(Message::SettingsShowChartSelectionTimeToggled)
                                    } else {
                                        None
                                    }
                                )
                                .style(style::fur_checkbox_style),
                            checkbox(self.fur_settings.show_chart_selection_earnings)
                                .label(
                                    self.localization
                                        .get_message("earnings-for-selection", None)
                                )
                                .on_toggle_maybe(
                                    if self.fur_settings.show_chart_breakdown_by_selection {
                                        Some(Message::SettingsShowChartSelectionEarningsToggled)
                                    } else {
                                        None
                                    }
                                )
                                .style(style::fur_checkbox_style),
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                // MARK: SETTINGS DATA TAB
                .push(
                    TabId::Data,
                    TabLabel::IconText(
                        advanced_text::floppy_fill().0.chars().next().unwrap_or(' '),
                        self.localization.get_message("data", None)
                    ),
                    Scrollable::new(
                        column![
                            settings_heading(self.localization.get_message("sync", None)),
                            sync_server_col,
                            settings_heading(self.localization.get_message("local-database", None)),
                            database_location_col,
                            settings_heading("CSV".to_string()),
                            csv_col,
                            settings_heading(self.localization.get_message("more", None)),
                            backup_col,
                        ]
                        .spacing(SETTINGS_SPACING)
                        .padding(10),
                    ),
                )
                .set_active_tab(&self.settings_active_tab)
                .tab_bar_position(TabBarPosition::Top)
        ];

        // MARK: INSPECTOR
        let inspector: Column<'_, Message, Theme, Renderer> = match &self.inspector_view {
            // MARK: Add Task To Group
            Some(FurInspectorView::AddNewTask) => match &self.task_to_add {
                Some(task_to_add) => column![
                    text_input(
                        &self.localization.get_message("task-name", None),
                        &task_to_add.name
                    )
                    .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Name))
                    .on_submit_maybe(if task_to_add.name.trim().is_empty() {
                        None
                    } else {
                        Some(Message::SaveTaskEdit)
                    }),
                    text_input(
                        &self.localization.get_message("project", None),
                        &task_to_add.project
                    )
                    .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Project))
                    .on_submit_maybe(if task_to_add.name.trim().is_empty() {
                        None
                    } else {
                        Some(Message::SaveTaskEdit)
                    }),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &task_to_add.tags
                    )
                    .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags))
                    .on_submit_maybe(if task_to_add.name.trim().is_empty() {
                        None
                    } else {
                        Some(Message::SaveTaskEdit)
                    }),
                    text_input("0.00", &task_to_add.new_rate)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Rate))
                        .on_submit_maybe(if task_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveTaskEdit)
                        }),
                    row![
                        text(self.localization.get_message("start-colon", None)),
                        date_picker(
                            task_to_add.show_start_date_picker,
                            task_to_add.displayed_start_date,
                            button(text(task_to_add.displayed_start_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StartDate),
                        ),
                        time_picker(
                            task_to_add.show_start_time_picker,
                            task_to_add.displayed_start_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_start_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StartTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("stop-colon", None)),
                        date_picker(
                            task_to_add.show_stop_date_picker,
                            task_to_add.displayed_stop_date,
                            button(text(task_to_add.displayed_stop_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StopDate),
                        ),
                        time_picker(
                            task_to_add.show_stop_time_picker,
                            task_to_add.displayed_stop_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_stop_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StopTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTaskEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::primary)
                        .on_press_maybe(if task_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveTaskEdit)
                        })
                        .width(Length::Fill)
                        .style(style::primary_button_style),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&task_to_add.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            // Add todo
            Some(FurInspectorView::AddNewTodo) => match &self.todo_to_add {
                Some(todo_to_add) => column![
                    text_input(
                        &self.localization.get_message("task", None),
                        &todo_to_add.name
                    )
                    .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Task))
                    .on_submit_maybe(if todo_to_add.name.trim().is_empty() {
                        None
                    } else {
                        Some(Message::SaveTodoEdit)
                    }),
                    text_input(
                        &self.localization.get_message("project", None),
                        &todo_to_add.project
                    )
                    .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Project))
                    .on_submit_maybe(if todo_to_add.name.trim().is_empty() {
                        None
                    } else {
                        Some(Message::SaveTodoEdit)
                    }),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &todo_to_add.tags
                    )
                    .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Tags))
                    .on_submit_maybe(if todo_to_add.name.trim().is_empty() {
                        None
                    } else {
                        Some(Message::SaveTodoEdit)
                    }),
                    text_input("0.00", &todo_to_add.rate)
                        .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Rate))
                        .on_submit_maybe(if todo_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveTodoEdit)
                        }),
                    row![
                        text(self.localization.get_message("date-colon", None)),
                        date_picker(
                            todo_to_add.show_date_picker,
                            todo_to_add.displayed_date,
                            button(text(todo_to_add.displayed_date.to_string()))
                                .on_press(Message::ChooseTodoEditDate)
                                .style(style::primary_button_style),
                            Message::CancelTodoEditDate,
                            |date| Message::SubmitTodoEditDate(date),
                        ),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTodoEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::primary)
                        .on_press_maybe(if todo_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveTodoEdit)
                        })
                        .width(Length::Fill)
                        .style(style::primary_button_style),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            // Add shortcut
            Some(FurInspectorView::AddShortcut) => match &self.shortcut_to_add {
                Some(shortcut_to_add) => column![
                    text(self.localization.get_message("new-shortcut", None)).size(24),
                    text_input(
                        &self.localization.get_message("task-name", None),
                        &shortcut_to_add.name
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Name))
                    .on_submit_maybe(
                        if shortcut_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        }
                    ),
                    text_input(
                        &self.localization.get_message("project", None),
                        &shortcut_to_add.project
                    )
                    .on_input(|s| {
                        Message::EditShortcutTextChanged(s, EditTaskProperty::Project)
                    })
                    .on_submit_maybe(
                        if shortcut_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        }
                    ),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &shortcut_to_add.tags
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Tags))
                    .on_submit_maybe(
                        if shortcut_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        }
                    ),
                    row![
                        text("$"),
                        text_input("0.00", &shortcut_to_add.new_rate)
                            .on_input(|s| {
                                Message::EditShortcutTextChanged(s, EditTaskProperty::Rate)
                            })
                            .on_submit_maybe(if shortcut_to_add.name.trim().is_empty() {
                                None
                            } else {
                                Some(Message::SaveShortcut)
                            }),
                        text(self.localization.get_message("per-hour", None)),
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center),
                    color_picker(
                        shortcut_to_add.show_color_picker,
                        shortcut_to_add.color,
                        button(
                            text(self.localization.get_message("color", None))
                                .style(|_| if is_dark_color(shortcut_to_add.color.to_srgb()) {
                                    text::Style {
                                        color: Some(Color::WHITE),
                                    }
                                } else {
                                    text::Style {
                                        color: Some(Color::BLACK),
                                    }
                                })
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press(Message::ChooseShortcutColor)
                        .width(Length::Fill)
                        .style(|theme, status| {
                            style::shortcut_button_style(
                                theme,
                                status,
                                shortcut_to_add.color.to_srgb(),
                            )
                        }),
                        Message::CancelShortcutColor,
                        Message::SubmitShortcutColor,
                    ),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelShortcut)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press_maybe(if shortcut_to_add.name.trim().is_empty() {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        })
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&shortcut_to_add.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            Some(FurInspectorView::AddTaskToGroup) => match &self.task_to_add {
                Some(task_to_add) => column![
                    text_input(&task_to_add.name, ""),
                    text_input(&task_to_add.project, ""),
                    text_input(&task_to_add.tags, ""),
                    text_input(&format!("{:.2}", task_to_add.rate), ""),
                    row![
                        text(self.localization.get_message("start-colon", None)),
                        button(
                            text(task_to_add.displayed_start_date.to_string())
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press_maybe(None)
                        .style(style::primary_button_style),
                        time_picker(
                            task_to_add.show_start_time_picker,
                            task_to_add.displayed_start_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_start_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StartTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("stop-colon", None)),
                        button(
                            text(task_to_add.displayed_stop_date.to_string())
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press_maybe(None)
                        .style(style::primary_button_style),
                        time_picker(
                            task_to_add.show_stop_time_picker,
                            task_to_add.displayed_stop_time,
                            button(
                                text(format_iced_time_as_hm(task_to_add.displayed_stop_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StopTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTaskEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press(Message::SaveTaskEdit)
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&task_to_add.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            // MARK: Edit Shortcut
            Some(FurInspectorView::EditShortcut) => match &self.shortcut_to_edit {
                Some(shortcut_to_edit) => column![
                    text(self.localization.get_message("edit-shortcut", None)).size(24),
                    text_input(
                        &self.localization.get_message("task-name", None),
                        &shortcut_to_edit.new_name
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Name))
                    .on_submit_maybe(
                        if shortcut_to_edit.new_name.trim().is_empty()
                            || !shortcut_to_edit.is_changed()
                        {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        }
                    ),
                    text_input(
                        &self.localization.get_message("project", None),
                        &shortcut_to_edit.new_project
                    )
                    .on_input(|s| {
                        Message::EditShortcutTextChanged(s, EditTaskProperty::Project)
                    })
                    .on_submit_maybe(
                        if shortcut_to_edit.new_name.trim().is_empty()
                            || !shortcut_to_edit.is_changed()
                        {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        }
                    ),
                    text_input(
                        &self.localization.get_message("hashtag-tags", None),
                        &shortcut_to_edit.new_tags
                    )
                    .on_input(|s| Message::EditShortcutTextChanged(s, EditTaskProperty::Tags))
                    .on_submit_maybe(
                        if shortcut_to_edit.new_name.trim().is_empty()
                            || !shortcut_to_edit.is_changed()
                        {
                            None
                        } else {
                            Some(Message::SaveShortcut)
                        }
                    ),
                    row![
                        text("$"),
                        text_input("0.00", &shortcut_to_edit.new_rate)
                            .on_input(|s| {
                                Message::EditShortcutTextChanged(s, EditTaskProperty::Rate)
                            })
                            .on_submit_maybe(
                                if shortcut_to_edit.new_name.trim().is_empty()
                                    || !shortcut_to_edit.is_changed()
                                {
                                    None
                                } else {
                                    Some(Message::SaveShortcut)
                                }
                            ),
                        text(self.localization.get_message("per-hour", None)),
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center),
                    color_picker(
                        shortcut_to_edit.show_color_picker,
                        shortcut_to_edit.new_color,
                        button(
                            text(self.localization.get_message("color", None))
                                .style(|_| if is_dark_color(shortcut_to_edit.new_color.to_srgb()) {
                                    text::Style {
                                        color: Some(Color::WHITE),
                                    }
                                } else {
                                    text::Style {
                                        color: Some(Color::BLACK),
                                    }
                                })
                                .width(Length::Fill)
                                .align_x(alignment::Horizontal::Center)
                        )
                        .on_press(Message::ChooseShortcutColor)
                        .width(Length::Fill)
                        .style(|theme, status| {
                            style::shortcut_button_style(
                                theme,
                                status,
                                shortcut_to_edit.new_color.to_srgb(),
                            )
                        }),
                        Message::CancelShortcutColor,
                        Message::SubmitShortcutColor,
                    ),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelShortcut)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press_maybe(
                            if shortcut_to_edit.new_name.trim().is_empty()
                                || !shortcut_to_edit.is_changed()
                            {
                                None
                            } else {
                                Some(Message::SaveShortcut)
                            }
                        )
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&shortcut_to_edit.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(INSPECTOR_SPACING)
                    .padding(INSPECTOR_PADDING)
                    .width(INSPECTOR_WIDTH)
                    .align_x(INSPECTOR_ALIGNMENT),
            },
            // MARK: Edit Single Task
            Some(FurInspectorView::EditTask) => match &self.task_to_edit {
                Some(task_to_edit) => column![
                    row![
                        button(bootstrap::x_lg())
                            .on_press(Message::CancelTaskEdit)
                            .style(button::text),
                        space::horizontal(),
                        button(bootstrap::trash_fill())
                            .on_press(if self.fur_settings.show_delete_confirmation {
                                Message::ShowAlert(FurAlert::DeleteTaskConfirmation)
                            } else {
                                Message::DeleteTasks
                            })
                            .style(button::text),
                    ],
                    text_input(&task_to_edit.name, &task_to_edit.new_name)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Name))
                        .on_submit_maybe(
                            if task_to_edit.is_changed() && !task_to_edit.new_name.trim().is_empty()
                            {
                                Some(Message::SaveTaskEdit)
                            } else {
                                None
                            }
                        ),
                    text_input(&task_to_edit.project, &task_to_edit.new_project)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Project))
                        .on_submit_maybe(
                            if task_to_edit.is_changed() && !task_to_edit.new_name.trim().is_empty()
                            {
                                Some(Message::SaveTaskEdit)
                            } else {
                                None
                            }
                        ),
                    text_input(&task_to_edit.tags, &task_to_edit.new_tags)
                        .on_input(|s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags))
                        .on_submit_maybe(
                            if task_to_edit.is_changed() && !task_to_edit.new_name.trim().is_empty()
                            {
                                Some(Message::SaveTaskEdit)
                            } else {
                                None
                            }
                        ),
                    row![
                        text("$"),
                        text_input(
                            &format!("{:.2}", &task_to_edit.rate),
                            &task_to_edit.new_rate
                        )
                        .on_input(|s| { Message::EditTaskTextChanged(s, EditTaskProperty::Rate) })
                        .on_submit_maybe(
                            if task_to_edit.is_changed() && !task_to_edit.new_name.trim().is_empty()
                            {
                                Some(Message::SaveTaskEdit)
                            } else {
                                None
                            }
                        ),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("start-colon", None)),
                        date_picker(
                            task_to_edit.show_displayed_start_date_picker,
                            task_to_edit.displayed_start_date,
                            button(text(task_to_edit.displayed_start_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StartDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StartDate),
                        ),
                        time_picker(
                            task_to_edit.show_displayed_start_time_picker,
                            task_to_edit.displayed_start_time,
                            Button::new(
                                text(format_iced_time_as_hm(task_to_edit.displayed_start_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StartTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StartTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StartTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("stop-colon", None)),
                        date_picker(
                            task_to_edit.show_displayed_stop_date_picker,
                            task_to_edit.displayed_stop_date,
                            button(text(task_to_edit.displayed_stop_date.to_string()))
                                .on_press(Message::ChooseTaskEditDateTime(
                                    EditTaskProperty::StopDate
                                ))
                                .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopDate),
                            |date| Message::SubmitTaskEditDate(date, EditTaskProperty::StopDate),
                        ),
                        time_picker(
                            task_to_edit.show_displayed_stop_time_picker,
                            task_to_edit.displayed_stop_time,
                            button(
                                text(format_iced_time_as_hm(task_to_edit.displayed_stop_time))
                                    .center()
                            )
                            .on_press(Message::ChooseTaskEditDateTime(EditTaskProperty::StopTime))
                            .width(Length::Fill)
                            .style(style::primary_button_style),
                            Message::CancelTaskEditDateTime(EditTaskProperty::StopTime),
                            |time| Message::SubmitTaskEditTime(time, EditTaskProperty::StopTime),
                        )
                        .use_24h(),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTaskEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(style::primary_button_style)
                        .on_press_maybe(
                            if task_to_edit.is_changed() && !task_to_edit.new_name.trim().is_empty()
                            {
                                Some(Message::SaveTaskEdit)
                            } else {
                                None
                            }
                        )
                        .width(Length::Fill),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                    text(&task_to_edit.invalid_input_error_message).style(style::red_text),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![].width(INSPECTOR_WIDTH),
            },
            // MARK:: Edit Group
            Some(FurInspectorView::EditGroup) => match &self.group_to_edit {
                Some(group_to_edit) => {
                    let mut group_info_column: Column<'_, Message, Theme, Renderer> =
                        column![text(&group_to_edit.name).font(font::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        }),]
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                        .spacing(5)
                        .padding(20);
                    if !group_to_edit.project.is_empty() {
                        group_info_column = group_info_column.push(text(&group_to_edit.project));
                    }
                    if !group_to_edit.tags.is_empty() {
                        group_info_column =
                            group_info_column.push(text!("#{}", group_to_edit.tags));
                    }
                    if group_to_edit.rate != 0.0 {
                        group_info_column =
                            group_info_column.push(text!("${}", &group_to_edit.rate));
                    }
                    let tasks_column: Scrollable<'_, Message, Theme, Renderer> =
                        Scrollable::new(group_to_edit.tasks.iter().fold(
                            Column::new().spacing(5),
                            |column, task| {
                                column
                                    .push(
                                        button(
                                            Container::new(column![
                                                text(
                                                    self.localization.get_message(
                                                        "start-to-stop",
                                                        Some(&HashMap::from([
                                                            (
                                                                "start",
                                                                FluentValue::from(
                                                                    task.start_time
                                                                        .format("%H:%M")
                                                                        .to_string()
                                                                )
                                                            ),
                                                            (
                                                                "stop",
                                                                FluentValue::from(
                                                                    task.stop_time
                                                                        .format("%H:%M")
                                                                        .to_string()
                                                                )
                                                            )
                                                        ]))
                                                    )
                                                )
                                                .font(font::Font {
                                                    weight: iced::font::Weight::Bold,
                                                    ..Default::default()
                                                }),
                                                text(self.localization.get_message(
                                                    "total-time-dynamic",
                                                    Some(&HashMap::from([(
                                                        "time",
                                                        FluentValue::from(
                                                            seconds_to_formatted_duration(
                                                                task.total_time_in_seconds(),
                                                                self.fur_settings.show_seconds
                                                            )
                                                        )
                                                    )]))
                                                ))
                                            ])
                                            .width(Length::Fill)
                                            .padding([5, 8])
                                            .style(style::group_edit_task_row),
                                        )
                                        .on_press(Message::EditTask(task.clone()))
                                        .style(button::text),
                                    )
                                    .padding(Padding {
                                        top: 0.0,
                                        right: 10.0,
                                        bottom: 10.0,
                                        left: 10.0,
                                    })
                            },
                        ));
                    column![
                        row![
                            button(bootstrap::x_lg())
                                .on_press(Message::CancelGroupEdit)
                                .style(button::text),
                            space::horizontal(),
                            button(if group_to_edit.is_in_edit_mode {
                                bootstrap::pencil()
                            } else {
                                bootstrap::pencil_fill()
                            })
                            .on_press_maybe(if group_to_edit.is_in_edit_mode {
                                None
                            } else {
                                Some(Message::ToggleGroupEditor)
                            })
                            .style(button::text),
                            button(bootstrap::plus_lg())
                                .on_press_maybe(if group_to_edit.is_in_edit_mode {
                                    None
                                } else {
                                    Some(Message::AddTaskToGroup(group_to_edit.clone()))
                                })
                                .style(button::text),
                            button(bootstrap::trash_fill())
                                .on_press(if self.fur_settings.show_delete_confirmation {
                                    Message::ShowAlert(FurAlert::DeleteGroupConfirmation)
                                } else {
                                    Message::DeleteTasks
                                })
                                .style(button::text),
                        ]
                        .padding(INSPECTOR_PADDING)
                        .width(INSPECTOR_WIDTH)
                        .spacing(5),
                        // .spacing(5),
                        match group_to_edit.is_in_edit_mode {
                            true => column![
                                text_input(&group_to_edit.name, &group_to_edit.new_name).on_input(
                                    |s| Message::EditTaskTextChanged(s, EditTaskProperty::Name)
                                ),
                                text_input(&group_to_edit.project, &group_to_edit.new_project)
                                    .on_input(|s| Message::EditTaskTextChanged(
                                        s,
                                        EditTaskProperty::Project
                                    )),
                                text_input(&group_to_edit.tags, &group_to_edit.new_tags).on_input(
                                    |s| Message::EditTaskTextChanged(s, EditTaskProperty::Tags)
                                ),
                                row![
                                    text("$"),
                                    text_input(
                                        &format!("{:.2}", &group_to_edit.rate),
                                        &group_to_edit.new_rate
                                    )
                                    .on_input(|s| {
                                        Message::EditTaskTextChanged(s, EditTaskProperty::Rate)
                                    }),
                                ]
                                .align_y(Alignment::Center)
                                .spacing(5),
                                row![
                                    button(
                                        text(self.localization.get_message("cancel", None))
                                            .align_x(alignment::Horizontal::Center)
                                    )
                                    .style(button::secondary)
                                    .on_press(Message::ToggleGroupEditor)
                                    .width(Length::Fill),
                                    button(
                                        text(self.localization.get_message("save", None))
                                            .align_x(alignment::Horizontal::Center)
                                    )
                                    .style(style::primary_button_style)
                                    .on_press_maybe(
                                        if group_to_edit.is_changed()
                                            && !group_to_edit.new_name.trim().is_empty()
                                        {
                                            Some(Message::SaveGroupEdit)
                                        } else {
                                            None
                                        }
                                    )
                                    .width(Length::Fill),
                                ]
                                .padding(Padding {
                                    top: 20.0,
                                    right: 0.0,
                                    bottom: 0.0,
                                    left: 0.0,
                                })
                                .spacing(10),
                            ]
                            .padding(20)
                            .spacing(5),
                            false => group_info_column,
                        },
                        tasks_column,
                    ]
                    .spacing(5)
                    .align_x(Alignment::Start)
                }
                None => column![text(
                    self.localization.get_message("nothing-selected", None)
                )]
                .spacing(12)
                .padding(20)
                .align_x(Alignment::Start),
            },
            // MARK: Edit Todo
            Some(FurInspectorView::EditTodo) => match &self.todo_to_edit {
                Some(todo_to_edit) => column![
                    row![
                        space::horizontal(),
                        button(bootstrap::trash_fill())
                            .on_press(Message::DeleteTodoPressed(todo_to_edit.uid.clone()))
                            .style(button::text),
                    ],
                    text_input(&todo_to_edit.name, &todo_to_edit.new_name)
                        .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Task))
                        .on_submit_maybe(
                            if todo_to_edit.name.trim().is_empty() || !todo_to_edit.is_changed() {
                                None
                            } else {
                                Some(Message::SaveTodoEdit)
                            }
                        ),
                    text_input(&todo_to_edit.project, &todo_to_edit.new_project)
                        .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Project))
                        .on_submit_maybe(
                            if todo_to_edit.name.trim().is_empty() || !todo_to_edit.is_changed() {
                                None
                            } else {
                                Some(Message::SaveTodoEdit)
                            }
                        ),
                    text_input(&todo_to_edit.tags, &todo_to_edit.new_tags)
                        .on_input(|s| Message::EditTodoTextChanged(s, EditTodoProperty::Tags))
                        .on_submit_maybe(
                            if todo_to_edit.name.trim().is_empty() || !todo_to_edit.is_changed() {
                                None
                            } else {
                                Some(Message::SaveTodoEdit)
                            }
                        ),
                    row![
                        text("$"),
                        text_input(
                            &format!("{:.2}", &todo_to_edit.rate),
                            &todo_to_edit.new_rate
                        )
                        .on_input(|s| { Message::EditTodoTextChanged(s, EditTodoProperty::Rate) })
                        .on_submit_maybe(
                            if todo_to_edit.name.trim().is_empty() || !todo_to_edit.is_changed() {
                                None
                            } else {
                                Some(Message::SaveTodoEdit)
                            }
                        ),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        text(self.localization.get_message("date-colon", None)),
                        date_picker(
                            todo_to_edit.show_date_picker,
                            todo_to_edit.displayed_date,
                            button(text(todo_to_edit.displayed_date.to_string()))
                                .on_press(Message::ChooseTodoEditDate)
                                .style(style::primary_button_style),
                            Message::CancelTodoEditDate,
                            |date| Message::SubmitTodoEditDate(date),
                        ),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5),
                    row![
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::secondary)
                        .on_press(Message::CancelTodoEdit)
                        .width(Length::Fill),
                        button(
                            text(self.localization.get_message("save", None))
                                .align_x(alignment::Horizontal::Center)
                        )
                        .style(button::primary)
                        .on_press_maybe(
                            if todo_to_edit.name.trim().is_empty() || !todo_to_edit.is_changed() {
                                None
                            } else {
                                Some(Message::SaveTodoEdit)
                            }
                        )
                        .width(Length::Fill)
                        .style(style::primary_button_style),
                    ]
                    .padding(Padding {
                        top: 20.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                    })
                    .spacing(10),
                ]
                .spacing(INSPECTOR_SPACING)
                .padding(INSPECTOR_PADDING)
                .width(INSPECTOR_WIDTH)
                .align_x(INSPECTOR_ALIGNMENT),
                None => column![]
                    .spacing(12)
                    .padding(20)
                    .width(250)
                    .align_x(Alignment::Start),
            },
            _ => column![],
        };

        let inspector_row = if self.inspector_view.is_some() {
            Some(row![rule::vertical(1), inspector].width(260))
        } else {
            None
        };

        let content = row![
            sidebar,
            // Main view
            match self.current_view {
                FurView::Shortcuts => shortcuts_view,
                FurView::Timer => timer_view,
                FurView::Todo => todo_view,
                FurView::Report => charts_view,
                FurView::Settings => settings_view,
            },
            inspector_row,
        ];

        let overlay: Option<Card<'_, Message, Theme, Renderer>> = if self.displayed_alert.is_some()
        {
            let alert_text: String;
            let alert_description: String;
            let close_button: Option<Button<'_, Message, Theme, Renderer>>;
            let mut confirmation_button: Option<Button<'_, Message, Theme, Renderer>> = None;
            let mut snooze_button: Option<Button<'_, Message, Theme, Renderer>> = None;

            match self.displayed_alert.as_ref().unwrap() {
                FurAlert::AutosaveRestored => {
                    alert_text = self.localization.get_message("autosave-restored", None);
                    alert_description = self
                        .localization
                        .get_message("autosave-restored-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("ok", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::primary),
                    );
                }
                FurAlert::DeleteEverythingConfirmation => {
                    alert_text = self
                        .localization
                        .get_message("delete-everything-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-everything-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete-everything", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteEverything)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteGroupConfirmation => {
                    alert_text = self.localization.get_message("delete-all-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-all-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete-all", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTasks)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteShortcutConfirmation => {
                    alert_text = self
                        .localization
                        .get_message("delete-shortcut-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-shortcut-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteShortcut)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteTaskConfirmation => {
                    alert_text = self.localization.get_message("delete-task-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-task-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTasks)
                        .style(button::danger),
                    );
                }
                FurAlert::DeleteTodoConfirmation => {
                    alert_text = self.localization.get_message("delete-todo-question", None);
                    alert_description = self
                        .localization
                        .get_message("delete-todo-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("cancel", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("delete", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::DeleteTodo)
                        .style(button::danger),
                    );
                }
                FurAlert::Idle => {
                    alert_text = self.localization.get_message(
                        "idle-alert-title",
                        Some(&HashMap::from([(
                            "duration",
                            FluentValue::from(self.idle.duration()),
                        )])),
                    );
                    alert_description = self
                        .localization
                        .get_message("idle-alert-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("continue", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::IdleReset)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("discard", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::IdleDiscard)
                        .style(button::danger),
                    );
                }
                FurAlert::ImportMacDatabase => {
                    alert_text = self.localization.get_message("import-old-database", None);
                    alert_description = self
                        .localization
                        .get_message("import-old-database-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("dont-import", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("import", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::ImportOldMacDatabase)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::NotifyOfSync => {
                    alert_text = self.localization.get_message("syncing-now-available", None);
                    alert_description = self.localization.get_message("syncing-now-possible", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("ok", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::NotifyOfSyncClose)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("learn-more", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::LearnAboutSync)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::PomodoroBreakOver => {
                    alert_text = self.localization.get_message("break-over-title", None);
                    alert_description = self
                        .localization
                        .get_message("break-over-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("stop", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStopAfterBreak)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(self.localization.get_message("continue", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroContinueAfterBreak)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::PomodoroOver => {
                    alert_text = self.localization.get_message("pomodoro-over-title", None);
                    alert_description = self
                        .localization
                        .get_message("pomodoro-over-description", None);
                    snooze_button = Some(
                        button(
                            text(self.localization.get_message(
                                "snooze-button",
                                Some(&HashMap::from([(
                                    "duration",
                                    FluentValue::from(self.fur_settings.pomodoro_snooze_length),
                                )])),
                            ))
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Shrink),
                        )
                        .on_press(Message::PomodoroSnooze)
                        .style(button::secondary),
                    );
                    close_button = Some(
                        button(
                            text(self.localization.get_message("stop", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStop)
                        .style(button::secondary),
                    );
                    confirmation_button = Some(
                        button(
                            text(
                                if self.fur_settings.pomodoro_extended_breaks
                                    && self.pomodoro.sessions
                                        % self.fur_settings.pomodoro_extended_break_interval
                                        == 0
                                {
                                    self.localization.get_message("long-break", None)
                                } else {
                                    self.localization.get_message("break", None)
                                },
                            )
                            .align_x(alignment::Horizontal::Center)
                            .width(Length::Fill),
                        )
                        .on_press(Message::PomodoroStartBreak)
                        .style(style::primary_button_style),
                    );
                }
                FurAlert::ShortcutExists => {
                    alert_text = self.localization.get_message("shortcut-exists", None);
                    alert_description = self
                        .localization
                        .get_message("shortcut-exists-description", None);
                    close_button = Some(
                        button(
                            text(self.localization.get_message("ok", None))
                                .align_x(alignment::Horizontal::Center)
                                .width(Length::Fill),
                        )
                        .on_press(Message::AlertClose)
                        .style(style::primary_button_style),
                    );
                }
            }

            let mut buttons: Row<'_, Message, Theme, Renderer> =
                row![].spacing(10).padding(5).width(Length::Fill);
            if let Some(more) = snooze_button {
                buttons = buttons.push(more);
            }
            if let Some(close) = close_button {
                buttons = buttons.push(close);
            }
            if let Some(confirmation) = confirmation_button {
                buttons = buttons.push(confirmation);
            }

            Some(
                Card::new(text(alert_text), text(alert_description))
                    .foot(buttons)
                    .max_width(if self.displayed_alert == Some(FurAlert::PomodoroOver) {
                        400.0
                    } else {
                        300.0
                    })
                    .style(style::fur_card),
            )
        } else {
            None
        };

        if let Some(alert) = overlay {
            modal(content, container(alert)).into()
        } else {
            content.into()
        }
    }
}

fn nav_button<'a>(nav_text: String, destination: FurView, active: bool) -> Button<'a, Message> {
    button(text(nav_text))
        .padding([5, 15])
        .on_press(Message::NavigateTo(destination))
        .width(Length::Fill)
        .style(if active {
            style::active_nav_menu_button_style
        } else {
            style::inactive_nav_menu_button_style
        })
}

fn history_group_row<'a, 'loc>(
    task_group: &'a FurTaskGroup,
    timer_is_running: bool,
    settings: &'a FurSettings,
    localization: &'loc Localization,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'loc>, Message> {
    let mut task_details_column: Column<'_, Message, Theme, Renderer> =
        column![text(&task_group.name).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),]
        .width(Length::FillPortion(6));
    if settings.show_task_project && !task_group.project.is_empty() {
        task_details_column = task_details_column.push(text!("@{}", task_group.project));
    }
    if settings.show_task_tags && !task_group.tags.is_empty() {
        task_details_column = task_details_column.push(text!("#{}", task_group.tags));
    }

    let mut task_row: Row<'_, Message, Theme, Renderer> =
        row![].align_y(Alignment::Center).spacing(5);
    if task_group.tasks.len() > 1 {
        task_row = task_row.push(
            Container::new(text(task_group.tasks.len()))
                .align_x(alignment::Horizontal::Center)
                .width(30)
                .style(style::group_count_circle),
        );
    }

    let total_time_str =
        seconds_to_formatted_duration(task_group.total_time, settings.show_seconds);
    let mut totals_column: Column<'_, Message, Theme, Renderer> =
        column![text(total_time_str).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })]
        .align_x(Alignment::End);

    if settings.show_task_earnings && task_group.rate > 0.0 {
        let total_earnings = task_group.rate * (task_group.total_time as f32 / 3600.0);
        totals_column = totals_column.push(text!("${:.2}", total_earnings));
    }

    let task_group_string = task_group.to_string();

    task_row = task_row.push(task_details_column);
    task_row = task_row.push(space::horizontal().width(Length::Fill));
    task_row = task_row.push(totals_column);
    task_row = task_row.push(
        button(bootstrap::arrow_repeat())
            .on_press_maybe(if timer_is_running {
                None
            } else {
                Some(Message::RepeatLastTaskPressed(task_group_string.clone()))
            })
            .style(button::text),
    );

    let history_row_button = button(
        Container::new(task_row)
            .padding([10, 15])
            .width(Length::Fill)
            .style(style::task_row),
    )
    .on_press(Message::EditGroup(task_group.clone()))
    .style(button::text);

    let task_group_ids = task_group.all_task_ids();
    let task_group_clone = task_group.clone();

    ContextMenu::new(
        history_row_button,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button(text(localization.get_message("repeat", None)))
                    .on_press(Message::RepeatLastTaskPressed(task_group_string.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("edit", None)))
                    .on_press(Message::EditGroup(task_group_clone.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("create-shortcut", None)))
                    .on_press(Message::CreateShortcutFromTaskGroup(
                        task_group_clone.clone(),
                    ))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("delete", None)))
                    .on_press(Message::DeleteTasksFromContext(task_group_ids.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
            ])
            .max_width(150)
            .into()
        }),
    )
}

fn history_title_row<'a>(
    date: &NaiveDate,
    total_time: i64,
    total_earnings: f32,
    settings: &FurSettings,
    running_timer: Option<(bool, &str, f32)>,
    localization: &Localization,
) -> Row<'a, Message> {
    let mut total_time_column = column![].align_x(Alignment::End);

    if settings.show_daily_time_total {
        let total_time = if settings.dynamic_total
            && let Some((true, timer_text, _)) = running_timer
        {
            seconds_to_formatted_duration(
                combine_timer_with_seconds(timer_text, total_time),
                settings.show_seconds,
            )
        } else {
            seconds_to_formatted_duration(total_time, settings.show_seconds)
        };
        total_time_column = total_time_column.push(text(total_time).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }));
    }

    if settings.show_task_earnings {
        let total_earnings = if settings.dynamic_total
            && let Some((true, timer_text, rate)) = running_timer
        {
            let timer_seconds = parse_timer_text_to_seconds(timer_text);
            total_earnings + ((timer_seconds as f32 / 3600.0) * rate)
        } else {
            total_earnings
        };
        if total_earnings > 0.0 {
            total_time_column = total_time_column.push(text!("${:.2}", total_earnings));
        }
    }

    row![
        text(format_history_date(date, localization)).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        space::horizontal().width(Length::Fill),
        total_time_column,
    ]
    .align_y(Alignment::Center)
}

fn format_history_date(date: &NaiveDate, localization: &Localization) -> String {
    let today = Local::now().date_naive();
    let yesterday = today - TimeDelta::days(1);
    let current_year = today.year();

    if date == &today {
        localization.get_message("today", None)
    } else if date == &yesterday {
        localization.get_message("yesterday", None)
    } else if date.year() == current_year {
        date.format("%b %d").to_string()
    } else {
        date.format("%b %d, %Y").to_string()
    }
}

fn shortcut_button_content<'a>(
    shortcut: &'a FurShortcut,
    text_color: Color,
) -> Column<'a, Message, Theme, Renderer> {
    let mut shortcut_text_column = column![
        text(&shortcut.name)
            .font(font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .style(move |_| text::Style {
                color: Some(text_color)
            })
    ]
    .spacing(5);

    if !shortcut.project.is_empty() {
        shortcut_text_column =
            shortcut_text_column.push(text!("@{}", shortcut.project).style(move |_| text::Style {
                color: Some(text_color),
            }));
    }
    if !shortcut.tags.is_empty() {
        shortcut_text_column =
            shortcut_text_column.push(text(&shortcut.tags).style(move |_| text::Style {
                color: Some(text_color),
            }));
    }
    if shortcut.rate > 0.0 {
        shortcut_text_column = shortcut_text_column.push(space::vertical());
        shortcut_text_column = shortcut_text_column.push(row![
            space::horizontal(),
            text!("${:.2}", shortcut.rate).style(move |_| text::Style {
                color: Some(text_color)
            })
        ]);
    }

    shortcut_text_column
}

fn shortcut_button<'a, 'loc>(
    shortcut: &'a FurShortcut,
    timer_is_running: bool,
    localization: &'loc Localization,
) -> ContextMenu<'a, Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'loc>, Message> {
    let shortcut_color = match Srgb::from_hex(&shortcut.color_hex) {
        Ok(color) => color,
        Err(_) => Srgb::new(0.694, 0.475, 0.945),
    };
    let text_color = if is_dark_color(shortcut_color) {
        Color::WHITE
    } else {
        Color::BLACK
    };

    let shortcut_button = button(shortcut_button_content(&shortcut, text_color))
        .width(200)
        .padding(10)
        .height(170)
        .on_press_maybe(if timer_is_running {
            None
        } else {
            Some(Message::ShortcutPressed(shortcut.to_string()))
        })
        .style(move |theme, status| style::shortcut_button_style(theme, status, shortcut_color));

    let shortcut_clone = shortcut.clone();

    ContextMenu::new(
        shortcut_button,
        Box::new(move || -> Element<'a, Message, Theme, Renderer> {
            Container::new(column![
                iced::widget::button(text(localization.get_message("edit", None)))
                    .on_press(Message::EditShortcutPressed(shortcut_clone.clone()))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
                iced::widget::button(text(localization.get_message("delete", None)))
                    .on_press(Message::DeleteShortcutFromContext(
                        shortcut_clone.uid.clone()
                    ))
                    .style(style::context_menu_button_style)
                    .width(Length::Fill),
            ])
            .max_width(150)
            .into()
        }),
    )
}

fn is_dark_color(color: Srgb) -> bool {
    color.relative_luminance().luma < 0.6
}

fn convert_timer_text_to_vertical_hms(timer_text: &str, localization: &Localization) -> String {
    let mut split = timer_text.split(':');
    let mut sidebar_timer_text = String::new();

    if let Some(hours) = split.next() {
        if hours != "0" {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-h",
                Some(&HashMap::from([("hours", FluentValue::from(hours))])),
            ));
            sidebar_timer_text.push_str("\n");
        }
    }

    if let Some(mins) = split.next() {
        if mins != "00" {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-m",
                Some(&HashMap::from([(
                    "minutes",
                    FluentValue::from(mins.trim_start_matches('0')),
                )])),
            ));
            sidebar_timer_text.push_str("\n");
        }
    }

    if let Some(secs) = split.next() {
        if secs != "00" {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-s",
                Some(&HashMap::from([(
                    "seconds",
                    FluentValue::from(secs.trim_start_matches('0')),
                )])),
            ));
        } else {
            sidebar_timer_text.push_str(&localization.get_message(
                "x-s",
                Some(&HashMap::from([("seconds", FluentValue::from("0"))])),
            ));
        }
    }

    sidebar_timer_text
}

fn settings_heading<'a>(heading: String) -> Column<'a, Message, Theme, Renderer> {
    column![
        text(heading).font(font::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Container::new(rule::horizontal(1)).max_width(200.0)
    ]
    .padding(Padding {
        top: 15.0,
        right: 0.0,
        bottom: 5.0,
        left: 0.0,
    })
}

fn combine_timer_with_seconds(timer: &str, seconds: i64) -> i64 {
    let timer_seconds = parse_timer_text_to_seconds(timer);
    timer_seconds + seconds
}

fn parse_timer_text_to_seconds(timer_text: &str) -> i64 {
    let parts: Vec<&str> = timer_text.split(':').collect();
    match parts.len() {
        3 => {
            let hours: i64 = parts[0].parse().unwrap_or(0);
            let minutes: i64 = parts[1].parse().unwrap_or(0);
            let seconds: i64 = parts[2].parse().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        }
        2 => {
            let minutes: i64 = parts[0].parse().unwrap_or(0);
            let seconds: i64 = parts[1].parse().unwrap_or(0);
            minutes * 60 + seconds
        }
        1 => parts[0].parse().unwrap_or(0),
        _ => 0,
    }
}

pub fn write_furtasks_to_csv(
    path: PathBuf,
    localization: &Localization,
) -> Result<(), Box<dyn std::error::Error>> {
    match std::fs::File::create(path) {
        Ok(file) => {
            match db_retrieve_all_existing_tasks(SortBy::StartTime, SortOrder::Descending) {
                Ok(tasks) => {
                    let mut csv_writer = Writer::from_writer(file);
                    csv_writer.write_record(&[
                        "Name",
                        "Start Time",
                        "Stop Time",
                        "Tags",
                        "Project",
                        "Rate",
                        "Currency",
                        "Total Time",
                        "Total Earnings",
                    ])?;

                    for task in tasks {
                        csv_writer.write_record(&[
                            task.name.clone(),
                            task.start_time.to_rfc3339(),
                            task.stop_time.to_rfc3339(),
                            task.tags.clone(),
                            task.project.clone(),
                            task.rate.to_string(),
                            task.currency.clone(),
                            seconds_to_formatted_duration(task.total_time_in_seconds(), true),
                            format!("${:.2}", task.total_earnings()),
                        ])?;
                    }

                    csv_writer.flush()?;
                    Ok(())
                }
                _ => Err(localization
                    .get_message("error-retrieving-tasks", None)
                    .into()),
            }
        }
        _ => Err(localization.get_message("error-creating-file", None).into()),
    }
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    alert: Container<'a, Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            center(opaque(row![
                space::horizontal(),
                alert,
                space::horizontal()
            ]))
            .style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            })
        )
    ]
    .into()
}

fn format_iced_time_as_hm(time: iced_aw::time_picker::Time) -> String {
    let naive_time = NaiveTime::from(time);
    naive_time.format("%H:%M").to_string()
}
