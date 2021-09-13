use std::str::FromStr;
use std::fmt;

#[derive(Eq, PartialEq, Ord, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "days_of_week")]
#[sqlx(rename_all = "lowercase")]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl FromStr for Day {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "Пн" => Ok(Day::Monday),
            "Вт" => Ok(Day::Tuesday),
            "Ср" => Ok(Day::Wednesday),
            "Чт" => Ok(Day::Thursday),
            "Пт" => Ok(Day::Friday),
            "Сб" => Ok(Day::Saturday),
            "Вс" => Ok(Day::Sunday),
            _ => Err("Вы ввели неправильный формат дня недели!"),
        }
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Day::Monday => {
                    "Понедельник"
                }
                Day::Tuesday => {
                    "Вторник"
                }
                Day::Wednesday => {
                    "Среда"
                }
                Day::Thursday => {
                    "Четверг"
                }
                Day::Friday => {
                    "Пятница"
                }
                Day::Saturday => {
                    "Суббота"
                }
                Day::Sunday => {
                    "Воскресенье"
                }
            }
        )
    }
}