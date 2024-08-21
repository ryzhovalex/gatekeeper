use std::time::{SystemTime, UNIX_EPOCH};

pub type Time = f64;

pub fn utc() -> Time {
    let time = SystemTime::now();
    time.duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
}

pub fn delta(d: Time) -> Time {
    let time = utc();
    time + d
}
