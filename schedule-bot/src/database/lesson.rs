use std::fmt;
use sqlx::types::chrono::NaiveTime;
use std::collections::BTreeMap;

use crate::database::Day;


#[derive(sqlx::Type)]
#[sqlx(type_name = "lesson_types")]
#[sqlx(rename_all = "lowercase")]
pub enum LessonType {
    Lecture,
    Practice,
    #[sqlx(rename = "laboratory work")]
    LaboratoryWork,
}

impl fmt::Display for LessonType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LessonType::Lecture => {
                    "лекція"
                }
                LessonType::Practice => {
                    "практика"
                }
                LessonType::LaboratoryWork => {
                    "лабораторна робота"
                }
            }
        )
    }
}

#[derive(sqlx::FromRow)]
pub struct Lesson {
    pub subject_name: String,
    pub lesson_type: LessonType,
    pub time: NaiveTime,
    pub teacher_name: String,
    pub info: String,
}

impl fmt::Display for Lesson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}): [{}] \nВикладач: {} \n{}",
            self.subject_name,
            self.lesson_type,
            self.time.format("%R"),
            self.teacher_name,
            self.info
        )
    }
}

pub struct Lessons(pub Vec<Lesson>);

impl fmt::Display for Lessons {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "Выходной день")
        } else {
            self.0.iter().fold(Ok(()), |result, lesson| {
                result.and_then(|_| writeln!(f, "\n{}", lesson))
            })
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct LessonDay {
    #[sqlx(rename = "day_of_week")]
    pub day: Day,
    pub subject_name: String,
    pub lesson_type: LessonType,
    pub time: NaiveTime,
    pub teacher_name: String,
    pub info: String,
}

pub struct LessonsWeek(pub BTreeMap<Day, Lessons>);

impl fmt::Display for LessonsWeek {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, day| {
            result.and_then(|_| writeln!(f, "{}\n{}", day.0, day.1))
        })
    }
}