pub mod cli;
pub mod logger;
pub mod list;
pub mod entry;
pub mod archive;
pub mod moc;
pub use logger::*;

pub fn home_dir() -> std::path::PathBuf {
    // Linux only; change this if you want to go cross platform
    match std::env::var("HOME") {
        Ok(path) => std::path::Path::new(&path).join(".diary-cli"),
        Err(_) => std::path::PathBuf::from("/etc/diary-cli/"),
    }
}

use chrono::{NaiveDate, Duration};
fn get_days_since_2020(date: [u16; 3]) -> Option<i64> {
    let input_date = match NaiveDate::from_ymd_opt(date[0] as i32, date[1] as u32, date[2] as u32) {
        Some(x) => x,
        None => return None,
    };
    let start_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let duration: Duration = input_date.signed_duration_since(start_date);
    Some(duration.num_days())
}