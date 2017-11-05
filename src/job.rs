use std::time::SystemTime;
use std::cmp::Ordering;
use times;

/*
The "Job" type - max possible values: u64::max_value() = 18446744073709551615.
internal_id will overflow after max value - internal functioning should not be affected.
*/
#[derive(Debug)]
pub struct Job {
    pub internal_id: u64,
    external_id: u64,
    trigger_at_ms: u64,
    body: String,
}

impl Job {
    pub fn new(internal_id: u64, external_id: u64, trigger_at_ms: u64, body: &str) -> Job {
        let body = body.to_owned();
        Job {
            internal_id,
            external_id,
            trigger_at_ms,
            body,
        }
    }

    pub fn new_without_external_id(internal_id: u64, trigger_at_ms: u64, body: &str) -> Job {
        let body = body.to_owned();
        let external_id = 0u64;
        Job {
            internal_id,
            external_id,
            trigger_at_ms,
            body,
        }
    }

    #[inline]
    pub fn trigger_at(&self) -> SystemTime {
        times::ms_to_system_time(self.trigger_at_ms)
    }

    #[inline]
    pub fn trigger_at_ms(&self) -> u64 {
        self.trigger_at_ms
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        self.trigger_at_ms <= times::current_time_ms()
    }
}

impl Ord for Job {
    /// A Job is greater than another job if the job's trigger time is further out in the future
    fn cmp(&self, other: &Job) -> Ordering {
        // Flip ordering - we want min heap
        // Close trigger time means job > further trigger_at time.
        self.trigger_at_ms.cmp(&other.trigger_at_ms).reverse()
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
        if self.external_id == 0 || other.external_id == 0 {
            return self.internal_id == other.internal_id;
        }
        self.internal_id == other.internal_id || self.external_id == other.external_id
    }
}

#[cfg(test)]
mod tests {
    use super::Job;
    use std::cmp::Ordering;

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
            "Job: {:?} should eq: {:?} Same internal id, diff external id",
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
            "Job: {:?} should eq: {:?} Same external id, diff internal id",
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
            "Job: {:?} should eq: {:?} Same external id, same internal id, diff trigger_at",
            j_one,
            j_two
        )
    }

    #[test]
    fn eq_missing_external_id_equality() {
        let j_one = Job::new_without_external_id(100u64, 5u64, "foo one");
        let j_two = Job::new(100u64, 100u64, 6u64, "foo two");
        assert_eq!(
            j_one,
            j_two,
            "Job: {:?} should eq: {:?} Missing external id, same internal id",
            j_one,
            j_two
        )
    }

    #[test]
    fn neq_missing_external_id_equality() {
        let j_one = Job::new_without_external_id(100u64, 5u64, "foo one");
        let j_two = Job::new(200u64, 1030u64, 6u64, "foo two");
        assert_ne!(
            j_one,
            j_two,
            "Job: {:?} should neq: {:?} Same internal id, missing external id, diff trigger_at",
            j_one,
            j_two
        )
    }

    #[test]
    fn job_ordering_test() {
        let one = Job::new(1, 1, 1, "one");
        let two = Job::new(2, 2, 2, "two");
        assert!(
            one > two,
            "A job with trigger time close to in the future is smaller in ordering"
        );

        let one = Job::new(1, 1, 2, "one");
        let two = Job::new(2, 2, 1, "two");
        assert!(
            one < two,
            "A job with trigger time close to in the future is greater in ordering"
        );

        let one = Job::new(1, 1, 2, "one");
        let two = Job::new(2, 2, 2, "two");
        assert_eq!(
            one.cmp(&two),
            Ordering::Equal,
            "When two jobs have same trigger_at time, ordering comparison is Equal"
        );
        assert!(
            one.ne(&two),
            "When two jobs have same trigger_at time, equality comparison is not affected"
        );
    }
}
