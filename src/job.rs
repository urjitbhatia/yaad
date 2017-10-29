use std::time::{SystemTime, Duration};
use std::ops::Add;
use std::{iter, thread};
/*
The "Job" type - max possible values: u64::max_value() = 18446744073709551615.
internal_id will overflow after max value - internal functioning should not be affected.
*/
//#[derive(Serialize, Deserialize, Debug)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Job {
    pub internal_id: u64,
    external_id: u64,
    pub trigger_at: SystemTime,
    body: String,
}

impl Job {
    pub fn new(internal_id: u64, external_id: u64, trigger_at_ms: u64, body: String) -> Job {
        Job {
            internal_id: internal_id,
            external_id: external_id,
            trigger_at: SystemTime::now().add(Duration::from_millis(trigger_at_ms)),
            body: body,
        }
    }

    pub fn is_ready(self) -> bool {
        self.trigger_at <= SystemTime::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_create_job() {
        let j = Job::new(100u64, 150u64, 5u64, "Test Body".to_owned());
        assert!(j.internal_id == 100u64);
    }
}
