use chrono::{DateTime, Utc};

pub fn get_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d").to_string()
}

pub fn get_nelo_time(date: &str) -> String {
    chrono::NaiveDate::parse_from_str(date, "%b %d,%y").unwrap().format("%Y-%m-%d").to_string()
}

pub fn get_duration(start: String, end: String) -> i32 {
    let start = chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
    let duration = end - start;
    duration.num_days() as i32
}