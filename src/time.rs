use std::time::{SystemTime, UNIX_EPOCH};

pub fn utc() -> f64 {
    let time = SystemTime::now();
    time.duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
}

pub fn delta(d: f64) -> f64 {
    let time = utc();
    time + d
}
