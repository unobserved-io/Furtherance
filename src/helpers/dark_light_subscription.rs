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
