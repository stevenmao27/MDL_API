use chrono::{DateTime, Utc};

pub fn get_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d").to_string()
}

pub fn get_duration(start: String, end: String) -> i32 {
    let start = start.parse::<DateTime<Utc>>().unwrap();
    let end = end.parse::<DateTime<Utc>>().unwrap();
    let duration = end - start;
    duration.num_days() as i32
}