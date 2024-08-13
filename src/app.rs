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

use crate::database::*;
use crate::fur_task::FurTask;
use crate::style::*;
use iced::{
    alignment, font, keyboard,
    multi_window::Application,
    widget::{
        button, column, horizontal_space, pick_list, row, text, text_input, theme, vertical_space,
        Column, Container, Scrollable,
    },
    window, Alignment, Command, Element, Font, Length, Renderer, Settings, Size, Theme,
};
use iced_aw::{
    core::icons::{bootstrap, BOOTSTRAP_FONT_BYTES},
    date_picker::{self, Date},
    modal, Card, Modal,
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
    current_view: FurView,
    show_modal: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),
    ModalClose,
    NavigateTo(FurView),
}

impl Application for Furtherance {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut furtherance = Furtherance {
            current_view: FurView::Timer,
            show_modal: false,
        };

        // Load or create database
        // match db_init() {
        //     Ok(_) => {}
        //     Err(e) => eprintln!("Error loading database: {}", e),
        // }

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
            Message::FontLoaded(_) => Command::none(),
            Message::ModalClose => Command::none(),
            Message::NavigateTo(destination) => {
                self.current_view = destination;
                Command::none()
            }
        }
    }

    fn view(&self, _window_id: window::Id) -> Element<Message> {
        // MARK: SIDEBAR
        let sidebar = column![
            button("Shortcuts")
                .on_press(Message::NavigateTo(FurView::Shortcuts))
                .style(theme::Button::Text),
            button("Timer")
                .on_press(Message::NavigateTo(FurView::Timer))
                .style(theme::Button::Text),
            button("History")
                .on_press(Message::NavigateTo(FurView::History))
                .style(theme::Button::Text),
            button("Report")
                .on_press(Message::NavigateTo(FurView::Report))
                .style(theme::Button::Text),
            vertical_space().height(Length::Fill),
            // TODO: if timer is running and nav is not timer, show timer
            button("Settings")
                .on_press(Message::NavigateTo(FurView::Shortcuts))
                .style(theme::Button::Text),
        ]
        .spacing(12)
        .padding(20)
        .width(175)
        .align_items(Alignment::Start);

        // MARK: Shortcuts
        let shortcuts_view = column![];

        // MARK: TIMER
        let timer_view = column![];

        // MARK: HISTORY
        let history_view = column![];

        // MARK: REPORT
        let report_view = column![];

        // MARK: SETTINGS
        let settings_view = column![];

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
