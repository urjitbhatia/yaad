use std::time::{SystemTime, Duration};
use std::ops::Add;
use std::iter;
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

pub fn new_job(internal_id: u64, external_id: u64, body: String) -> Job {
    //! TODO - using a default 1/2 sec trigger time right now
    let trigger_at = SystemTime::now().add(Duration::from_millis(500));
    return Job {
        internal_id: internal_id,
        external_id: external_id,
        trigger_at: trigger_at,
        body: body,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_create_job() {
        let j = new_job(100u64, 150u64, "Test Body".to_owned());
        assert!(j.internal_id == 100u64);
    }
}
