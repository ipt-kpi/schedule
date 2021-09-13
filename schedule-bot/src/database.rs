use anyhow::Result;
use futures::future::BoxFuture;
use once_cell::sync::OnceCell;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool, Row};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{Serializer, Storage};


use crate::config::Config;
use crate::database::lesson::{Lessons, Lesson, LessonsWeek, LessonDay};
use crate::database::week::WeekType;
use crate::database::day::Day;

mod lesson;
pub mod week;
pub mod day;

static INSTANCE: OnceCell<Database<Json>> = OnceCell::new();

pub struct Database<S> {
    pool: PgPool,
    serializer: S,
}

pub async fn initialize(config: &Config) -> Result<()> {
    INSTANCE
        .set(Database {
            pool: PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&config.database_url)
                .await?,
            serializer: Json,
        })
        .map_err(|_| anyhow::anyhow!("Failed to initialize database!"))
}

impl<S, D> Storage<D> for &'static Database<S>
where
    S: Send + Sync + Serializer<D> + 'static,
    D: Send + Serialize + DeserializeOwned + 'static,
    <S as Serializer<D>>::Error: Debug + Display,
{
    type Error = anyhow::Error;

    fn remove_dialogue(
        self: Arc<Self>,
        chat_id: i64,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            Ok(match get_dialogue(&self.pool, chat_id).await? {
                Some(d) => {
                    let prev_dialogue = self.serializer.deserialize(&d).map_err(|error| {
                        anyhow::anyhow!("dialogue serialization error: {}", error)
                    })?;
                    sqlx::query("DELETE FROM teloxide_dialogues WHERE chat_id = $1")
                        .bind(chat_id)
                        .execute(&self.pool)
                        .await?;
                    Some(prev_dialogue)
                }
                _ => None,
            })
        })
    }

    fn update_dialogue(
        self: Arc<Self>,
        chat_id: i64,
        dialogue: D,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            let prev_dialogue = get_dialogue(&self.pool, chat_id)
                .await?
                .map(|d| {
                    self.serializer
                        .deserialize(&d)
                        .map_err(|error| anyhow::anyhow!("Database deserialize error: {}", error))
                })
                .transpose()?;
            let upd_dialogue = self
                .serializer
                .serialize(&dialogue)
                .map_err(|error| anyhow::anyhow!("Database serialize error: {}", error))?;
            self.pool
                .acquire()
                .await?
                .execute(
                    sqlx::query(
                        r#"
            INSERT INTO teloxide_dialogues VALUES ($1, $2)
            ON CONFLICT(chat_id) DO UPDATE SET dialogue=excluded.dialogue
                                "#,
                    )
                    .bind(chat_id)
                    .bind(upd_dialogue),
                )
                .await
                .unwrap();
            Ok(prev_dialogue)
        })
    }
}

async fn get_dialogue(pool: &PgPool, chat_id: i64) -> Result<Option<Box<Vec<u8>>>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct DialogueDbRow {
        dialogue: Vec<u8>,
    }

    Ok(sqlx::query_as::<_, DialogueDbRow>(
        "SELECT dialogue FROM teloxide_dialogues WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?
    .map(|r| Box::new(r.dialogue)))
}

impl Database<Json> {
    pub fn global() -> &'static Database<Json> {
        INSTANCE.get().expect("Pool is not initialized")
    }

    pub async fn get_schedule_by_day(&self, day: Day, user_id: i64) -> Result<Lessons> {
        sqlx::query_as::<_, Lesson>("SELECT * FROM get_schedule($1, $2)")
            .bind(day)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|lessons| Lessons(lessons))
    }

    pub async fn get_today_schedule(&self, user_id: i64) -> Result<Lessons> {
        sqlx::query_as::<_, Lesson>("SELECT * FROM get_today_schedule($1)")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|lessons| Lessons(lessons))
    }

    pub async fn get_current_schedule(&self, user_id: i64) -> Result<Lessons> {
        sqlx::query_as::<_, Lesson>("SELECT * FROM get_current_schedule($1)")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|lessons| Lessons(lessons))
    }

    pub async fn get_week_schedule(&self, user_id: i64, week: &WeekType) -> Result<LessonsWeek> {
        let lessons: Vec<LessonDay> =
            sqlx::query_as::<_, LessonDay>("SELECT * FROM get_week_schedule($1, $2)")
                .bind(user_id)
                .bind(week)
                .fetch_all(&self.pool)
                .await
                .map_err(|error| anyhow::anyhow!(error))?;
        let mut lessons_week = BTreeMap::new();
        for lesson in lessons {
            lessons_week
                .entry(lesson.day)
                .or_insert(Lessons(vec![]))
                .0
                .push(Lesson {
                    subject_name: lesson.subject_name,
                    lesson_type: lesson.lesson_type,
                    time: lesson.time,
                    teacher_name: lesson.teacher_name,
                    info: lesson.info,
                })
        }
        Ok(LessonsWeek(lessons_week))
    }

    pub async fn get_distribution_week(&self) -> Result<WeekType> {
        sqlx::query("SELECT * FROM get_distribution_week()")
            .fetch_one(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|row| row.get(0))
    }
}
