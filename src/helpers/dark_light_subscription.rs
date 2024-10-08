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

use futures::StreamExt;
use iced::advanced::subscription;

pub struct DarkLightSubscription;

impl subscription::Recipe for DarkLightSubscription {
    type Output = dark_light::Mode;

    fn hash(&self, state: &mut subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        input: iced::futures::stream::BoxStream<'static, subscription::Event>,
    ) -> iced::futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            (input, ()),
            |(input, _)| async move {
                match dark_light::subscribe().await {
                    Ok(mut stream) => {
                        let next = stream.next().await;
                        Some((next.unwrap_or(dark_light::Mode::Default), (input, ())))
                    }
                    Err(_) => None,
                }
            },
        ))
    }
}
