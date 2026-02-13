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

use std::sync::Arc;

use iced::theme::{Custom, Palette};
use iced::widget::{button, checkbox, container, text, toggler};
use iced::{Background, Border, Color, Gradient, Radians, Theme, gradient};
use iced_aw::card;
use iced_aw::style::number_input;
use palette::{Lighten, Srgb};

use crate::constants::FURTHERANCE_PURPLE;
use crate::helpers::color_utils::{ToIcedColor, ToSrgb};

pub struct FurPalette;

impl FurPalette {
    pub fn light() -> Palette {
        let mut palette = Palette::LIGHT;
        palette.primary = FURTHERANCE_PURPLE.to_iced_color();
        palette
    }

    pub fn dark() -> Palette {
        let mut palette = Palette::DARK;
        palette.primary = FURTHERANCE_PURPLE.to_iced_color();
        palette
    }
}

#[derive(Debug, Clone)]
pub enum FurTheme {
    Light,
    Dark,
}

impl FurTheme {
    pub fn to_theme(&self) -> Theme {
        match self {
            FurTheme::Light => Theme::Custom(Arc::new(Custom::new(
                "FurThemeLight".to_string(),
                FurPalette::light(),
            ))),
            FurTheme::Dark => Theme::Custom(Arc::new(Custom::new(
                "FurThemeDark".to_string(),
                FurPalette::dark(),
            ))),
        }
    }
}

#[allow(dead_code)]
trait ThemeExt {
    fn is_fur_theme_dark(&self) -> bool;
    fn is_fur_theme_light(&self) -> bool;
}

#[allow(dead_code)]
impl ThemeExt for Theme {
    fn is_fur_theme_dark(&self) -> bool {
        let palette = self.extended_palette();
        let bg = palette.background.base.color;

        // Calculate relative luminance to guess if we are in dark mode
        let (r, g, b) = (bg.r, bg.g, bg.b);
        (0.2126 * r + 0.7152 * g + 0.0722 * b) < 0.5
    }

    fn is_fur_theme_light(&self) -> bool {
        !self.is_fur_theme_dark()
    }
}

pub fn gray_background(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        ..Default::default()
    }
}

pub fn fur_card(theme: &Theme, _status: card::Status) -> card::Style {
    let palette = theme.extended_palette();

    card::Style {
        border_color: palette.primary.weak.color.into(),
        border_width: 3.0,
        head_background: palette.primary.weak.color.into(),
        ..Default::default()
    }
}

pub fn task_row(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn group_edit_task_row(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        border: Border {
            color: palette.background.weak.color.into(),
            width: 1.5,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn group_count_circle(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.strong.color.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 50.0.into(),
        },
        ..Default::default()
    }
}

pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.primary.base.color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            ..button::Style::default()
        },
        button::Status::Hovered => {
            let primary_color: Color = palette.primary.base.color;
            let light_color = primary_color.to_srgb().lighten(0.3).to_iced_color();
            button::Style {
                background: Some(light_color.into()),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            }
        }
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(palette.primary.base.color))
                .map(|background| background.scale_alpha(0.5)),
            text_color: Color::WHITE.scale_alpha(0.5),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            ..button::Style::default()
        },
    }
}

pub fn red_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::from_rgb8(190, 0, 0)),
    }
}

pub fn green_text(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::from_rgb8(0, 180, 0)),
    }
}

pub fn shortcut_button_style(
    _theme: &Theme,
    status: button::Status,
    primary_color: Srgb,
) -> button::Style {
    let light_color = primary_color.lighten(0.3);
    let primary_style = button::Style {
        background: Some(Background::Gradient(Gradient::Linear(
            gradient::Linear::new(Radians(std::f32::consts::PI))
                .add_stop(0.0, light_color.to_iced_color())
                .add_stop(1.0, primary_color.to_iced_color()),
        ))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 15.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Active | button::Status::Pressed => primary_style,
        button::Status::Hovered => {
            let lighter_color = light_color.lighten(0.3);
            button::Style {
                background: Some(Background::Gradient(Gradient::Linear(
                    gradient::Linear::new(Radians(std::f32::consts::PI))
                        .add_stop(0.0, lighter_color.to_iced_color())
                        .add_stop(1.0, light_color.to_iced_color()),
                ))),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 15.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::TRANSPARENT,
                    offset: iced::Vector { x: 0.0, y: 0.0 },
                    blur_radius: 0.0,
                },
                ..button::Style::default()
            }
        }
        _ => primary_style,
    }
}

pub fn context_menu_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let primary_color: Color = palette.primary.base.color;
    let light_color = primary_color.to_srgb().lighten(0.6).to_iced_color();

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Background::Color(light_color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Style::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.base.color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Style::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(light_color))
                .map(|background| background.scale_alpha(0.5)),
            text_color: Color::BLACK.scale_alpha(0.5),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Style::default()
        },
    }
}

pub fn active_nav_menu_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let active_nav_button_style = button::Style {
        background: Some(Background::Color(palette.primary.base.color)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 15.0.into(),
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => active_nav_button_style,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.base.color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 15.0.into(),
            },
            ..button::Style::default()
        },
        _ => active_nav_button_style,
    }
}

pub fn inactive_nav_menu_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let inactive_nav_button_style = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: if theme.is_fur_theme_dark() {
            Color::WHITE
        } else {
            Color::BLACK
        },
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => inactive_nav_button_style,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.primary.base.color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 15.0.into(),
            },
            ..button::Style::default()
        },
        _ => inactive_nav_button_style,
    }
}

pub fn fur_toggler_style(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let palette = theme.extended_palette();

    match status {
        toggler::Status::Active { is_toggled } => toggler::Style {
            background: if is_toggled {
                iced::Background::Color(palette.primary.base.color)
            } else {
                iced::Background::Color(palette.background.strong.color)
            },
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: iced::Background::Color(Color::WHITE),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            // TODO: Check if these fields need to be changed after upgrade to Iced 0.14
            text_color: None,
            border_radius: None,
            padding_ratio: 0.0,
        },
        toggler::Status::Hovered { is_toggled } => toggler::Style {
            background: if is_toggled {
                iced::Background::Color(palette.primary.base.color)
            } else {
                iced::Background::Color(palette.background.strong.color)
            },
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: iced::Background::Color(FURTHERANCE_PURPLE.lighten(0.3).to_iced_color()),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            // TODO: Check if these fields need to be changed after upgrade to Iced 0.14
            text_color: None,
            border_radius: None,
            padding_ratio: 0.0,
        },
        toggler::Status::Disabled { is_toggled: _ } => toggler::Style {
            background: iced::Background::Color(palette.background.strong.color),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: iced::Background::Color(palette.background.weak.color),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            // TODO: Check if these fields need to be changed after upgrade to Iced 0.14
            text_color: None,
            border_radius: None,
            padding_ratio: 0.0,
        },
    }
}

pub fn fur_checkbox_style(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let palette = theme.extended_palette();

    match status {
        checkbox::Status::Active { is_checked } => checkbox::Style {
            background: Background::Color(if is_checked {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            }),
            icon_color: Color::WHITE,
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.primary.base.color,
            },
            text_color: None,
        },
        checkbox::Status::Hovered { is_checked } => checkbox::Style {
            background: Background::Color(if is_checked {
                palette.primary.base.color
            } else {
                palette.background.weak.color
            }),
            icon_color: Color::WHITE,
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.primary.base.color,
            },
            text_color: None,
        },
        checkbox::Status::Disabled { is_checked } => checkbox::Style {
            background: Background::Color(if is_checked {
                palette.background.strong.color
            } else {
                palette.background.weak.color
            }),
            icon_color: Color::WHITE,
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
            text_color: None,
        },
    }
}

pub fn fur_number_input_style(
    theme: &Theme,
    status: iced_aw::style::Status,
) -> number_input::Style {
    let palette = theme.extended_palette();

    match status {
        iced_aw::style::Status::Active => number_input::Style {
            button_background: Some(palette.primary.base.color.into()),
            icon_color: Color::WHITE,
        },
        iced_aw::style::Status::Pressed => number_input::Style {
            button_background: Some(palette.primary.strong.color.into()),
            icon_color: Color::WHITE,
        },
        iced_aw::style::Status::Disabled => number_input::Style {
            button_background: Some(palette.background.strong.color.into()),
            icon_color: Color::WHITE,
        },
        _ => number_input::Style {
            button_background: Some(palette.primary.base.color.into()),
            icon_color: Color::WHITE,
        },
    }
}
