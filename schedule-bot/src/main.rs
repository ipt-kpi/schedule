use anyhow::Result;
use config::Config;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;
use teloxide::Bot;

use crate::database::Database;
use crate::dialogue::states::StartState;
use crate::dialogue::Dialogue;
use crate::schedule::command::Command;

mod config;
mod database;
mod dialogue;
mod schedule;

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting schedule_bot...");

    let config = Config::new().expect("Failed to initialize config!");
    database::initialize(&config).await.unwrap();

    let bot = Bot::new(config.token).auto_send();
    run(bot).await.expect("Something get wrong with main task");
}

type In = DialogueWithCx<AutoSend<Bot>, Message, Dialogue, anyhow::Error>;

async fn run(bot: AutoSend<Bot>) -> Result<()> {
    teloxide::enable_logging!();
    log::info!("Starting queue_bot...");

    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::with_storage(
            |DialogueWithCx { cx, dialogue }: In| async move {
                let dialogue = dialogue.expect("std::convert::Infallible");
                handle_message(cx, dialogue)
                    .await
                    .expect("Something wrong with the bot!")
            },
            Arc::new(Database::global()),
        ))
        .dispatch()
        .await;
    Ok(())
}

async fn handle_message(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    dialogue: Dialogue,
) -> TransitionOut<Dialogue> {
    match cx.update.text().map(ToOwned::to_owned) {
        None => {
            cx.answer("Отправьте текстовое сообщение").await?;
            next(dialogue)
        }
        Some(ans) => match ans.as_str() {
            "/start" => {
                if !dialogue.is_start() {
                    next(Dialogue::Start(StartState))
                } else {
                    (Command::Help).answer(&cx, dialogue).await
                }
            }
            ans => match Command::parse(ans, "") {
                Ok(command) => command.answer(&cx, dialogue).await,
                Err(_) => dialogue.react(cx, ans.to_string()).await,
            },
        },
    }
}
