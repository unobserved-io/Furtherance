use std::time::Duration;

use chrono::{Local, TimeDelta};
use iced::advanced::subscription;

use crate::app::Message;

pub struct MidnightSubscription;

impl subscription::Recipe for MidnightSubscription {
    type Output = Message;

    fn hash(&self, state: &mut rustc_hash::FxHasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> futures_core::stream::BoxStream<'static, Self::Output> {
        Box::pin(async_stream::stream! {
            loop {
                let now = Local::now();
                let next_midnight = (now + TimeDelta::days(1))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap();
                let duration_until_midnight = next_midnight - now;

                tokio::time::sleep(Duration::from_secs(duration_until_midnight.num_seconds() as u64)).await;

                yield Message::MidnightReached;
            }
        })
    }
}
