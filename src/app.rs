// jaCounter - Keep track of JustAnswer Expert earnings
// Copyright (C) 2024 Ricky Kresslein <ricky@unobserved.io>

use crate::database::*;
use crate::fur_task::FurTask;
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
    core::icons::bootstrap,
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
        match db_init() {
            Ok(_) => {}
            Err(e) => eprintln!("Error loading database: {}", e),
        }

        (
            furtherance,
            font::load(iced_aw::core::icons::BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
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
        }
    }

    fn view(&self, window_id: window::Id) -> Element<Message> {
        // MARK: TIMER
        let timer_view = column![];

        // MARK: TIMER
        let timer_view = column![];

        // MARK: HISTORY
        let history_view = column![];

        // MARK: REPORT
        let report_view = column![];

        // MARK: SETTINGS
        let settings_view = column![];

        let content = match self.current_view {
            FurView::Shortcuts => timer_view,
            FurView::Timer => timer_view,
            FurView::History => history_view,
            FurView::Report => report_view,
            FurView::Settings => settings_view,
        };

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
