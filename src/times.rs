use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ops::Add;

#[inline]
/// Returns current time in ms - drops `nanosec` precision
pub fn current_time_ms() -> u64 {
    system_time_to_ms(SystemTime::now())
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
    UNIX_EPOCH.add(Duration::from_millis(ms))
}

#[inline]
/// Returns given system time in ms - drops `nanosec` precision
pub fn system_time_to_ms(system_time: SystemTime) -> u64 {
    duration_to_ms(system_time.duration_since(UNIX_EPOCH).unwrap())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_system_time_conversion() {
        let now = SystemTime::now();

        let now_ms = system_time_to_ms(now);
        let now = ms_to_system_time(now_ms);
        let now_no_nanos_ms = system_time_to_ms(now);

        assert_eq!(now_ms, now_no_nanos_ms);
    }
}
