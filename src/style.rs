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

use std::sync::Arc;

use iced::theme::{self, Checkbox, Custom, Palette};
use iced::widget::{button, checkbox, container, toggler};
use iced::{gradient, Background, Border, Color, Gradient, Radians, Theme};
use iced_aw::style::number_input;
use palette::{Darken, Lighten, Srgb};

use crate::constants::FURTHERANCE_PURPLE;
use crate::helpers::color_utils::ToIcedColor;

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

impl From<FurTheme> for Theme {
    fn from(theme: FurTheme) -> Theme {
        let palette = match theme {
            FurTheme::Light => FurPalette::light(),
            FurTheme::Dark => FurPalette::dark(),
        };

        Theme::Custom(Arc::new(Custom::new(
            format!("FurTheme{:?}", theme),
            palette,
        )))
    }
}

pub fn gray_background(theme: &Theme) -> container::Appearance {
    let palette = theme.extended_palette();

    container::Appearance {
        background: Some(palette.background.weak.color.into()),
        ..Default::default()
    }
}

pub fn task_row(theme: &Theme) -> container::Appearance {
    let palette = theme.extended_palette();

    container::Appearance {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn group_edit_task_row(theme: &Theme) -> container::Appearance {
    let palette = theme.extended_palette();

    container::Appearance {
        border: Border {
            color: palette.background.weak.color.into(),
            width: 1.5,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn group_count_circle(theme: &Theme) -> container::Appearance {
    let palette = theme.extended_palette();

    container::Appearance {
        background: Some(palette.background.strong.color.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 50.0.into(),
        },
        ..Default::default()
    }
}

struct PrimaryButtonStyle {
    primary_color: Color,
    light_color: Color,
}

impl button::StyleSheet for PrimaryButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.primary_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.light_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            ..button::Appearance::default()
        }
    }
}

pub fn primary_button_style() -> iced::theme::Button {
    let primary_color = FURTHERANCE_PURPLE.to_iced_color();
    let light_color = FURTHERANCE_PURPLE.lighten(0.3).to_iced_color();
    iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
        primary_color,
        light_color,
    }))
}

struct ShortcutButtonStyle {
    primary_color: Srgb,
    light_color: Srgb,
}

impl button::StyleSheet for ShortcutButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Gradient(Gradient::Linear(
                gradient::Linear::new(Radians(std::f32::consts::PI))
                    .add_stop(0.0, self.light_color.to_iced_color())
                    .add_stop(1.0, self.primary_color.to_iced_color()),
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
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        let lighter_color = self.light_color.lighten(0.3);
        button::Appearance {
            background: Some(Background::Gradient(Gradient::Linear(
                gradient::Linear::new(Radians(std::f32::consts::PI))
                    .add_stop(0.0, lighter_color.to_iced_color())
                    .add_stop(1.0, self.light_color.to_iced_color()),
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
            ..button::Appearance::default()
        }
    }
}

pub fn shortcut_button_style(primary_color: Srgb) -> iced::theme::Button {
    let light_color = primary_color.lighten(0.3);
    iced::theme::Button::Custom(Box::new(ShortcutButtonStyle {
        primary_color,
        light_color,
    }))
}

struct ContextMenuButtonStyle {
    primary_color: Color,
    light_color: Color,
}

impl button::StyleSheet for ContextMenuButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.light_color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.primary_color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Appearance::default()
        }
    }
}

pub fn context_menu_button_style() -> iced::theme::Button {
    let light_color = FURTHERANCE_PURPLE.lighten(0.6).to_iced_color();
    let primary_color = FURTHERANCE_PURPLE.to_iced_color();
    iced::theme::Button::Custom(Box::new(ContextMenuButtonStyle {
        primary_color,
        light_color,
    }))
}

struct ActiveNavigationButtonStyle {
    primary_color: Color,
}

impl button::StyleSheet for ActiveNavigationButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.primary_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 15.0.into(),
            },
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.primary_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 15.0.into(),
            },
            ..button::Appearance::default()
        }
    }
}

pub fn active_nav_menu_button_style() -> iced::theme::Button {
    let primary_color = FURTHERANCE_PURPLE.to_iced_color();
    iced::theme::Button::Custom(Box::new(ActiveNavigationButtonStyle { primary_color }))
}

struct InactiveNavigationButtonStyle {
    primary_color: Color,
}

impl button::StyleSheet for InactiveNavigationButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: Color::BLACK,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.primary_color)),
            text_color: Color::WHITE,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 15.0.into(),
            },
            ..button::Appearance::default()
        }
    }
}

pub fn inactive_nav_menu_button_style() -> iced::theme::Button {
    let primary_color = FURTHERANCE_PURPLE.to_iced_color();
    iced::theme::Button::Custom(Box::new(InactiveNavigationButtonStyle { primary_color }))
}

struct FurTogglerStyle {}

impl toggler::StyleSheet for FurTogglerStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        let palette = Theme::extended_palette(style);
        toggler::Appearance {
            background: if is_active {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            },
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Color::WHITE,
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        let palette = Theme::extended_palette(style);
        toggler::Appearance {
            background: if is_active {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            },
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: FURTHERANCE_PURPLE.lighten(0.3).to_iced_color(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
        }
    }
}

pub fn fur_toggler_style() -> iced::theme::Toggler {
    iced::theme::Toggler::Custom(Box::new(FurTogglerStyle {}))
}

struct FurCheckboxStyle {}

impl checkbox::StyleSheet for FurCheckboxStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = Theme::extended_palette(style);
        checkbox::Appearance {
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
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = Theme::extended_palette(style);
        checkbox::Appearance {
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
        }
    }

    fn disabled(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = Theme::extended_palette(style);
        checkbox::Appearance {
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
        }
    }
}

pub fn fur_checkbox_style() -> iced::theme::Checkbox {
    iced::theme::Checkbox::Custom(Box::new(FurCheckboxStyle {}))
}

struct FurDisabledCheckboxStyle {}

impl checkbox::StyleSheet for FurDisabledCheckboxStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = Theme::extended_palette(style);
        checkbox::Appearance {
            background: Background::Color(if is_checked {
                palette.background.strong.color
            } else {
                palette.background.weak.color
            }),
            icon_color: Color::BLACK,
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.primary.strong.color,
            },
            text_color: None,
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = Theme::extended_palette(style);
        checkbox::Appearance {
            background: Background::Color(if is_checked {
                palette.background.strong.color
            } else {
                palette.background.weak.color
            }),
            icon_color: Color::BLACK,
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.primary.strong.color,
            },
            text_color: None,
        }
    }
}

pub fn fur_disabled_checkbox_style() -> iced::theme::Checkbox {
    iced::theme::Checkbox::Custom(Box::new(FurDisabledCheckboxStyle {}))
}

pub struct FurNumberInputStyle;

impl number_input::StyleSheet for FurNumberInputStyle {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> number_input::Appearance {
        let palette = Theme::extended_palette(style);
        number_input::Appearance {
            button_background: Some(palette.primary.base.color.into()),
            icon_color: Color::WHITE,
        }
    }

    fn pressed(&self, style: &Self::Style) -> number_input::Appearance {
        let palette = Theme::extended_palette(style);
        number_input::Appearance {
            button_background: Some(palette.primary.base.color.into()),
            icon_color: Color::WHITE,
        }
    }

    fn disabled(&self, style: &Self::Style) -> number_input::Appearance {
        let palette = Theme::extended_palette(style);
        number_input::Appearance {
            button_background: Some(palette.background.strong.color.into()),
            icon_color: Color::WHITE,
        }
    }
}
