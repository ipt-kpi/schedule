use serde::{Deserialize, Serialize};
use teloxide::macros::Transition;

use crate::dialogue::states::{DayState, StartState};

pub mod states;

#[derive(Transition, Serialize, Deserialize)]
pub enum Dialogue {
    Start(StartState),
    Day(DayState),
}

impl Dialogue {
    pub fn is_start(&self) -> bool {
        match &self {
            Dialogue::Start(_) => true,
            _ => false,
        }
    }
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}
