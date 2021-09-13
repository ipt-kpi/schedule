use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    _state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer("Неизвестная комманда, введите /help").await?;
    next(Dialogue::Start(StartState))
}
