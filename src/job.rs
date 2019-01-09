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
use temporal_state::{Temporal, TemporalState};
use times;
use uuid::{Uuid, UuidVersion};

///The "Job" type has max possible values: u64::max_value() = 18446744073709551615.
///internal_id will overflow after max value - internal functioning should not be affected.
#[derive(Debug)]
pub struct Job {
    job_metadata: JobMetadata,
    body: JobBody,
}

#[derive(Debug, Copy, Clone)]
pub struct JobMetadata {
    id: Uuid,
    trigger_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct JobBody {
    body: String,
}

impl Job {
    /// Creates a new job given an internal id, external id, trigger time in ms and the body.
    /// TODO: This does not handle id collisions properly yet.
    pub fn new(id: Uuid, trigger_at_ms: u64, body: &str) -> Job {
        match id.get_version() {
            Some(ver) => match ver {
                UuidVersion::Random => {
                    let body = body.to_owned();
                    Job {
                        job_metadata: JobMetadata { id, trigger_at_ms },
                        body: JobBody { body },
                    }
                }
                _ => panic!("Only uuid v4 ids are accepted"),
            },
            _ => panic!("Only uuid v4 ids are accepted"),
        }
    }

    pub fn new_from_metadata(job_metadata: JobMetadata, body: JobBody) -> Job {
        Job { job_metadata, body }
    }

    /// Creates new job that doesn't need an external id. An external id will not be generated in
    /// this case.
    pub fn new_auto_id(trigger_at_ms: u64, body: &str) -> Job {
        Job::new(Uuid::new_v4(), trigger_at_ms, body)
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
    pub fn get_body(&self) -> JobBody {
        self.body.clone()
    }

    #[inline]
    pub fn get_metadata(&self) -> JobMetadata {
        self.job_metadata.clone()
    }
}

impl JobMetadata {
    pub fn new(id: Uuid, trigger_at_ms: u64) -> JobMetadata {
        JobMetadata { id, trigger_at_ms }
    }

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

    #[inline]
    pub fn get_id(&self) -> (Uuid) {
        self.id.clone()
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
        self.id.eq(&other.id)
    }
}

impl Temporal for Job {
    fn as_temporal_state(&self) -> TemporalState {
        let now = times::current_time_ms();
        let delta = now - self.job_metadata.trigger_at_ms;
        if delta == 0 {
            return TemporalState::Current;
        } else if delta > 0 {
            return TemporalState::Future;
        }
        return TemporalState::Past;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_job() {
        let id = Uuid::new_v4();
        let j = Job::new(id, 5u64, "Test Body");
        assert_eq!(j.job_metadata.id, id, "Should be able to create a job");
    }

    #[test]
    fn id_equality() {
        let id = Uuid::new_v4();
        let j_one = Job::new(id, 100, "foo one");
        let j_two = Job::new(id, 100, "foo two");
        assert_eq!(
            j_one, j_two,
            "Job: {:?} should be eq: {:?} when ids are same",
            j_one, j_two
        )
    }

    #[test]
    fn job_ordering_test() {
        let one = Job::new_auto_id(1, "one");
        let two = Job::new_auto_id(2, "two");
        assert!(
            one > two,
            "A job with trigger time closer in the future is smaller in ordering"
        );

        let one = Job::new_auto_id(2, "one");
        let two = Job::new_auto_id(1, "two");
        assert!(
            one < two,
            "A job with trigger time closer in the future is greater in ordering"
        );

        let one = Job::new_auto_id(2, "one");
        let two = Job::new_auto_id(2, "two");
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
