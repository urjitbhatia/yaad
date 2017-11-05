use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::Sub;

#[inline]
pub fn current_time_ms() -> u64 {
    duration_to_ms(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
}

#[inline]
pub fn floor_ms_from_epoch(ms: u64) -> u64 {
    (ms / 10) * 10
}

#[inline]
pub fn duration_to_ms(d: Duration) -> u64 {
    (d.as_secs() * 1000) as u64 + (d.subsec_nanos() as u64 / 1_000_000)
}

#[inline]
pub fn ms_to_system_time(ms: u64) -> SystemTime {
    return UNIX_EPOCH.sub(Duration::from_millis(ms));
}
