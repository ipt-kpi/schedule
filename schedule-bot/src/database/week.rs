use std::str::FromStr;
use std::fmt;

#[derive(sqlx::Type)]
#[sqlx(type_name = "distribution_week")]
#[sqlx(rename_all = "lowercase")]
pub enum WeekType {
    First,
    Second,
}

impl FromStr for WeekType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "1" => Ok(WeekType::First),
            "2" => Ok(WeekType::Second),
            _ => Err("Введите 1 или 2 после команды"),
        }
    }
}

impl fmt::Display for WeekType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                WeekType::First => "первая",
                WeekType::Second => "вторая",
            }
        )
    }
}