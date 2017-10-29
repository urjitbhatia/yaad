use std::time::{SystemTime, Duration};
use std::ops::Add;
use std::cmp::Ordering;

/*
The "Job" type - max possible values: u64::max_value() = 18446744073709551615.
internal_id will overflow after max value - internal functioning should not be affected.
*/
//#[derive(Serialize, Deserialize, Debug)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub internal_id: u64,
    external_id: u64,
    trigger_at: SystemTime,
    body: String,
}

impl Job {
    pub fn new(internal_id: u64, external_id: u64, trigger_at_ms: u64, body: &str) -> Job {
        let trigger_at = SystemTime::now().add(Duration::from_millis(trigger_at_ms));
        let body = body.to_owned();
        Job {
            internal_id,
            external_id,
            trigger_at,
            body,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.trigger_at <= SystemTime::now()
    }
}

impl Ord for Job {
    /// A Job is greater than another job if the job's trigger time is further out in the future
    fn cmp(&self, other: &Job) -> Ordering {
        // Flip ordering - we want min heap (other.cmp(self)) rather than self.cmp(other)
        other.trigger_at.cmp(&self.trigger_at)
    }
}

impl Eq for Job {}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Job) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Job {
    /// A job's equality depends on the equality of either internal or external id
    fn eq(&self, other: &Job) -> bool {
        self.internal_id == other.internal_id || self.external_id == other.external_id
    }
}


#[cfg(test)]
mod tests {
    use super::Job;

    #[test]
    fn can_create_job() {
        let j = Job::new(100u64, 150u64, 5u64, "Test Body");
        assert_eq!(j.internal_id, 100u64);
    }

    #[test]
    fn internal_id_equality() {
        let j_one = Job::new(100u64, 200u64, 5u64, "foo one");
        let j_two = Job::new(100u64, 300u64, 5u64, "foo two");
        assert_eq!(
            j_one,
            j_two,
            "Job: {:?} should be equal to: {:?} Same internal id, diff external id",
            j_one,
            j_two
        )
    }

    #[test]
    fn external_id_equality() {
        let j_one = Job::new(100u64, 200u64, 5u64, "foo one");
        let j_two = Job::new(200u64, 200u64, 5u64, "foo two");
        assert_eq!(
            j_one,
            j_two,
            "Job: {:?} should be equal to: {:?} Same external id, diff internal id",
            j_one,
            j_two
        )
    }

    #[test]
    fn external_internal_id_equality() {
        let j_one = Job::new(100u64, 200u64, 5u64, "foo one");
        let j_two = Job::new(100u64, 200u64, 6u64, "foo two");
        assert_eq!(
            j_one,
            j_two,
            "Job: {:?} should be equal to: {:?} Same external id, same internal id, diff trigger_at",
            j_one,
            j_two
        )
    }
}
