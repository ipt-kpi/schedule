use serde::{Deserialize, Serialize};
use std::str::FromStr;
use teloxide::prelude::*;

use crate::database::Database;
use crate::database::day::Day;
use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct DayState;

#[teloxide(subtransition)]
async fn day(
    _state: DayState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    if let Ok(day) = Day::from_str(ans.as_ref()) {
        let msg = format!(
            "{}",
            Database::global()
                .get_schedule_by_day(day, cx.chat_id())
                .await
                .unwrap()
        );
        cx.answer(msg).send().await?;
        next(Dialogue::Day(DayState))
    } else {
        cx.answer("Неверно введен день недели").await?;
        return next(Dialogue::Day(DayState));
    }
}
