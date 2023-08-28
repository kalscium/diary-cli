use chrono::{NaiveDate, Duration, Utc};
use soulog::*;

pub fn get_days_since_2020(year: u16, month: u16, day: u16) -> Option<i64> {
    let input_date = match NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32) {
        Some(x) => x,
        None => return None,
    };
    let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let duration: Duration = input_date.signed_duration_since(start_date);
    Some(duration.num_days())
}

pub fn since_2023(date: Option<Vec<u16>>, mut logger: impl Logger) {
    match date {
        Some(date) => match get_days_since_2020(date[0], date[1], date[2]) {
            Some(x) => log!((logger) Since("{}{x}", colour_format![green("Days inbetween "), cyan("2020 "), green("and "), cyan(&date[2].to_string()), blue("/"), cyan(&date[1].to_string()), blue("/"), cyan(&date[0].to_string()), blue(": ")])),
            None => {
                log!((logger.error) Since("Invalid date provided") as Fatal);
                logger.crash()
            }
        },
        None => {
            let today = Utc::now().date_naive();
            let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
            let days = today.signed_duration_since(start_date).num_days();
            log!((logger) Since("{}{days}", colour_format![green("Days since "), cyan("2020"), blue(": ")]))
        }
    }
}