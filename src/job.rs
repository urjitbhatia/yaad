//! Jobs are the building blocks of the system - these are the individual items
//! that can be acted upon.
//!
//! Each Job has a `trigger` time - the time, in Milliseconds since EPOCH (UTC) when this
//! job is due.
//!
//! Jobs implement the PartialEquality trait [`job::PartialEq`] which orders them by nearness of
//! execution time: a job whose trigger time is closer in the future is `greater` than a job that
//! is due later.

use std::cmp::Ordering;
use times;

///The "Job" type has max possible values: u64::max_value() = 18446744073709551615.
///internal_id will overflow after max value - internal functioning should not be affected.
#[derive(Debug)]
pub struct Job {
    job_metadata: JobMetadata,
    body: JobBody,
}

#[derive(Debug)]
struct JobMetadata {
    pub internal_id: u64,
    external_id: u64,
    trigger_at_ms: u64,
}

#[derive(Debug)]
struct JobBody {
    body: String,
}

impl Job {
    /// Creates a new job given an internal id, external id, trigger time in ms and the body.
    /// TODO: This does not handle id collisions properly yet.
    pub fn new(internal_id: u64, external_id: u64, trigger_at_ms: u64, body: &str) -> Job {
        let body = body.to_owned();
        Job {
            job_metadata: JobMetadata {
                internal_id,
                external_id,
                trigger_at_ms,
            },

            body: JobBody { body },
        }
    }

    /// Creates new job that doesn't need an external id. An external id will not be generated in
    /// this case.
    pub fn new_without_external_id(internal_id: u64, trigger_at_ms: u64, body: &str) -> Job {
        Job::new(internal_id, 0, trigger_at_ms, body)
    }

    /// Returns the job's trigger time as milliseconds from UnixEpoch.
    #[inline]
    pub fn trigger_at_ms(&self) -> u64 {
        self.job_metadata.trigger_at_ms()
    }

    /// Returns true if the job should trigger right now.
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.job_metadata.is_ready()
    }

    #[inline]
    pub fn get_body(&self) -> String {
        self.body.body.clone()
    }
}

impl JobMetadata {
    /// Returns the job's trigger time as milliseconds from UnixEpoch.
    #[inline]
    pub fn trigger_at_ms(&self) -> u64 {
        self.trigger_at_ms
    }

    /// Returns true if the job should trigger right now.
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.trigger_at_ms <= times::current_time_ms()
    }
}

impl Ord for Job {
    /// A Job is greater than another job if the job's trigger time will happen before the other's
    fn cmp(&self, other: &Job) -> Ordering {
        // Flip ordering - we want min heap
        // Close trigger time means job > further trigger_at time.
        self.job_metadata.cmp(&other.job_metadata)
    }
}

impl Eq for Job {}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Job) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// PartialEq for a Job type ignores the `external_id` if it isn't set on either job being compared
impl PartialEq for Job {
    /// A job's equality depends on the equality of either internal or external id
    fn eq(&self, other: &Job) -> bool {
        self.job_metadata.eq(&other.job_metadata)
    }
}

impl Ord for JobMetadata {
    /// A Job is greater than another job if the job's trigger time will happen before the other's
    fn cmp(&self, other: &JobMetadata) -> Ordering {
        // Flip ordering - we want min heap
        // Close trigger time means job > further trigger_at time.
        self.trigger_at_ms.cmp(&other.trigger_at_ms).reverse()
    }
}

impl Eq for JobMetadata {}

impl PartialOrd for JobMetadata {
    fn partial_cmp(&self, other: &JobMetadata) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// PartialEq for a Job type ignores the `external_id` if it isn't set on either job being compared
impl PartialEq for JobMetadata {
    /// A job's equality depends on the equality of either internal or external id
    fn eq(&self, other: &JobMetadata) -> bool {
        if self.external_id == 0 || other.external_id == 0 {
            return self.internal_id == other.internal_id;
        }
        self.internal_id == other.internal_id || self.external_id == other.external_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_job() {
        let j = Job::new(100u64, 150u64, 5u64, "Test Body");
        assert_eq!(j.job_metadata.internal_id, 100u64, "Should be able to create a job");
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
            "A job with trigger time closer in the future is smaller in ordering"
        );

        let one = Job::new(1, 1, 2, "one");
        let two = Job::new(2, 2, 1, "two");
        assert!(
            one < two,
            "A job with trigger time closer in the future is greater in ordering"
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
