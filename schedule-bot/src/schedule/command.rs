use crate::database::Database;
use crate::database::week::WeekType;
use crate::dialogue::states::DayState;
use crate::dialogue::Dialogue;
use teloxide::dispatching::UpdateWithCx;
use teloxide::prelude::*;
use teloxide::types::{KeyboardButton, KeyboardMarkup, MessageKind};
use teloxide::utils::command::BotCommand;

#[derive(BotCommand)]
#[command(
    rename = "lowercase",
    description = "Список поддерживаемых комманд:",
    parse_with = "split"
)]
pub enum Command {
    #[command(description = "показать это сообщение.")]
    Help,
    #[command(description = "показать расписание на сегодня.")]
    Today,
    #[command(description = "показать расписание на выбранный день.")]
    Day,
    #[command(description = "показать текущую и последующую пары.")]
    Current,
    #[command(
        description = "показать расписание на выбранную неделю. (Введите 1 или 2 после команды)."
    )]
    Week(WeekType),
    #[command(description = "показать список дедлайнов.[WIP]")]
    Deadline,
    #[command(
        rename = "current_week",
        description = "показать какая неделя по счету."
    )]
    CurrentWeek,
}

impl Command {
    pub async fn answer(
        &self,
        cx: &UpdateWithCx<AutoSend<Bot>, Message>,
        dialogue: Dialogue,
    ) -> TransitionOut<Dialogue> {
        if let MessageKind::Common(msg) = &cx.update.kind {
            if let Some(user) = &msg.from {
                match self {
                    Command::Help => cx.answer(Command::descriptions()).send().await?,
                    Command::Day => {
                        cx.answer("Выберите день недели")
                            .reply_markup(days())
                            .send()
                            .await?;
                        return next(Dialogue::Day(DayState));
                    }
                    Command::Today => {
                        let msg = format!(
                            "{}",
                            Database::global()
                                .get_today_schedule(user.id)
                                .await
                                .unwrap()
                        );
                        cx.answer(msg).send().await?
                    }
                    Command::Current => {
                        let msg = format!(
                            "{}",
                            Database::global()
                                .get_current_schedule(user.id)
                                .await
                                .unwrap()
                        );
                        cx.answer(msg).send().await?
                    }
                    Command::Week(week) => {
                        let msg = format!(
                            "{}",
                            Database::global()
                                .get_week_schedule(user.id, week)
                                .await
                                .unwrap()
                        );
                        cx.answer(msg).send().await?
                    }
                    Command::CurrentWeek => {
                        let msg = format!(
                            "Сейчас {} неделя",
                            Database::global().get_distribution_week().await.unwrap()
                        );
                        cx.answer(msg).send().await?
                    }
                    _ => cx.answer(Command::descriptions()).send().await?,
                };
            }
        }
        next(dialogue)
    }
}

fn days() -> KeyboardMarkup {
    KeyboardMarkup::default()
        .append_row(vec![
            KeyboardButton::new("Пн"),
            KeyboardButton::new("Вт"),
            KeyboardButton::new("Ср"),
            KeyboardButton::new("Чт"),
            KeyboardButton::new("Пт"),
            KeyboardButton::new("Сб"),
            KeyboardButton::new("Вс"),
        ])
        .resize_keyboard(true)
}
