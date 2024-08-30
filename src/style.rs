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

use iced::widget::{button, container};
use iced::{gradient, theme, Background, Border, Color, Gradient, Radians, Theme};
use palette::color_difference::Wcag21RelativeContrast;
use palette::{Lighten, Srgb};

pub fn gray_background(theme: &Theme) -> container::Appearance {
    let palette = theme.extended_palette();

    container::Appearance {
        background: Some(palette.background.weak.color.into()),
        ..Default::default()
    }
}

fn palette_to_iced(color: Srgb) -> Color {
    let (r, g, b) = color.into_components();
    Color::from_rgb(r as f32, g as f32, b as f32)
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
                    .add_stop(0.0, palette_to_iced(self.light_color))
                    .add_stop(1.0, palette_to_iced(self.primary_color)),
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
                    .add_stop(0.0, palette_to_iced(lighter_color))
                    .add_stop(1.0, palette_to_iced(self.light_color)),
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

pub fn custom_button_style(primary_color: Srgb) -> iced::theme::Button {
    let light_color = primary_color.lighten(0.3);
    println!(
        "Base luminance: {:?}",
        primary_color.relative_luminance().luma
    );
    println!("Light luminance: {:?}", light_color.relative_luminance());
    iced::theme::Button::Custom(Box::new(ShortcutButtonStyle {
        primary_color,
        light_color,
    }))
}
