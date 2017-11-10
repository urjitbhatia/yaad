//! A Spoke is a list of jobs whose trigger times fall within the Spoke's duration of
//! responsibility.

use std::collections::{BinaryHeap, HashMap};
use std::collections::binary_heap::PeekMut;
use std::fmt;
use std::cmp::Ordering;
use times;
use uuid::Uuid;

// our module
use job::{Job, JobMetadata, JobBody};

/// A Spoke is a time-bound chain of jobs
///
/// Each spoke has a start time and a max Duration (inclusive)
/// Any job that should trigger in this time bound should be handled
/// by this spoke.
#[derive(Debug)]
pub struct Spoke {
    id: Uuid,
    bst: BoundingSpokeTime,
    job_id_map: HashMap<Uuid, JobBody>,
    job_list: BinaryHeap<JobMetadata>,
    // Todo rename to job_queue?
}

#[derive(Debug, Copy, Clone, Eq, Hash)]
pub struct BoundingSpokeTime {
    start_time_ms: u64,
    end_time_ms: u64,
}

impl BoundingSpokeTime {
    pub fn new(start_time_ms: u64, end_time_ms: u64) -> BoundingSpokeTime {
        BoundingSpokeTime {
            start_time_ms,
            end_time_ms,
        }
    }

    #[inline]
    pub fn get_start_time_ms(&self) -> u64 {
        self.start_time_ms
    }
    #[inline]
    pub fn get_end_time_ms(&self) -> u64 {
        self.end_time_ms
    }

    pub fn contains(&self, other: &BoundingSpokeTime) -> bool {
        self.start_time_ms <= other.start_time_ms && self.end_time_ms > other.end_time_ms
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        self.start_time_ms <= times::current_time_ms()
    }

    #[inline]
    pub fn is_expired(&self) -> bool {
        self.end_time_ms < times::current_time_ms()
    }
}

impl Spoke {
    /// Constructs a new Spoke - a time bound chain of jobs starting at the current time
    /// # Example
    /// Create a spoke that starts at 5 sec, 0 ns from now
    ///
    ///```
    /// use Spoke;
    /// let s = Spoke::new_from_now(Duration::new(5, 0));
    /// s.add_job(Job::new(1, 2, 3, "hi");
    ///```
    #[allow(dead_code)]
    fn new_from_now(duration_ms: u64) -> Spoke {
        Spoke::new(times::current_time_ms(), duration_ms)
    }

    pub fn new_from_bounds(bst: BoundingSpokeTime) -> Spoke {
        let job_id_map = HashMap::new();
        let job_list = BinaryHeap::new();
        let id = Uuid::new_v4();
        Spoke {
            id,
            bst,
            job_id_map,
            job_list,
        }
    }
    /// Constructs a new Spoke - a time bound chain of jobs starting at the current time
    /// # Example
    /// Create a spoke that starts at 5 sec, 0 ns from now
    ///
    ///```
    /// use Spoke;
    /// let s = Spoke::new(Duration::new(5, 0));
    /// s.add_job(Job::new(1, 2, 3, "hi");
    ///```
    pub fn new(start_time_ms: u64, duration_ms: u64) -> Spoke {
        let end_time_ms = start_time_ms + duration_ms;
        let bst = BoundingSpokeTime {
            start_time_ms,
            end_time_ms,
        };
        Spoke::new_from_bounds(bst)
    }

    /// Add a new job into the Spoke - the job is optionally returned if the Spoke is not the right
    /// one to take the job's responsibility.
    ///
    /// A Spoke is `responsible` for a job if that job's trigger time lies in the Spoke's
    /// time bounds
    pub fn add_job(&mut self, job: Job) -> Option<Job> {
        if self.is_expired() {
            return Option::from(job);
        }
        if self.bst.start_time_ms <= job.trigger_at_ms() &&
            job.trigger_at_ms() < self.bst.end_time_ms
        {
            // Only accept jobs that are this spoke's responsibility
            let jm = job.get_metadata();

            println!("Inserting jm: {:?}", jm);
            self.job_id_map.insert(jm.get_id(), job.get_body());
            self.job_list.push(jm);
            return Option::None;
        } else {
            // Return jobs that you don't want to accept
            return Option::from(job);
        }
    }

    /// Walk returns an iterator that returns jobs in trigger order
    ///
    /// Call walk in a loop like an iterator on this spoke
    /// # Example
    /// ```
    ///
    /// use Spoke;
    /// use times;
    ///
    /// let c = times::current_time_ms();
    /// let s = Spoke.new_from_now(Duration::from_millis(10_000, 0));
    /// s.add_job(Job::new(1, 1, c + 2500, "hello world");
    /// s.add_job(Job::new(2, 2, c + 5500, "hello world again");
    /// let i = s.walk().iter()
    /// for j in i {
    ///   println!("Job: {:?}", j)
    /// }
    /// ```
    pub fn walk(&mut self) -> Vec<Job> {
        let mut ready_jobs: Vec<Job> = vec![];

        while let Some(peeked) = self.job_list.peek_mut() {
            if peeked.is_ready() {
                let jm = PeekMut::pop(peeked);
                match self.job_id_map.remove(&jm.get_id()) {
                    Some(b) => ready_jobs.push(Job::new_from_metadata(jm, b)),
                    None => (),
                }
            } else {
                break;
            }
        }
        ready_jobs
    }

    pub fn cancel_job(&mut self, id: Uuid) -> bool {
        // Try to delete using internal id then
        match self.job_id_map.remove(&id) {
            Some(_) => true, // This does not remove from job list atm
            //when walking it will just not point to anything
            None => false,
        }
    }

    pub fn owns_job(&self, id: Uuid) -> bool {
        self.job_id_map.contains_key(&id)
    }

    /// Returns the number of jobs pending in this spoke
    #[inline]
    pub fn pending_job_len(&self) -> usize {
        self.job_list.len()
    }

    /// Returns true if this Spoke's start time is now or in the past
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.bst.is_ready()
    }

    #[inline]
    pub fn get_bounds(&self) -> BoundingSpokeTime {
        self.bst
    }

    /// Returns true if this Spoke's end time is in the past.
    ///
    /// If a job is in the `expired` state, it will not accept any new jobs. Jobs can only be taken
    /// from an expired Spoke.
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.bst.is_expired()
    }
}

impl fmt::Display for Spoke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Start time: {:?}, Duration: {:?} sec, NumJobs: {}, JobList: {:?})",
            self.bst.start_time_ms,
            self.bst.end_time_ms,
            self.job_list.len(),
            self.job_list
        )
    }
}

impl Ord for Spoke {
    /// A Spoke is greater than another spoke if it's start time is nearer in the future
    /// and it's end time is strictly less than the other's start time.
    fn cmp(&self, other: &Spoke) -> Ordering {
        // Flip ordering
        self.bst
            .start_time_ms
            .cmp(&other.bst.start_time_ms)
            .then(self.bst.end_time_ms.cmp(&other.bst.end_time_ms))
            .reverse()
    }
}

impl Eq for Spoke {}

impl PartialOrd for Spoke {
    fn partial_cmp(&self, other: &Spoke) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Spoke {
    fn eq(&self, other: &Spoke) -> bool {
        self.bst.start_time_ms.eq(&other.bst.start_time_ms) &
            &self.bst.end_time_ms.eq(&other.bst.end_time_ms)
    }
}

impl Ord for BoundingSpokeTime {
    /// A BoundingSpokeTime is greater than another spoke if it's start time is nearer in the future
    /// and it's end time is strictly less than the other's start time.
    fn cmp(&self, other: &BoundingSpokeTime) -> Ordering {
        // Flip ordering
        self.start_time_ms
            .cmp(&other.start_time_ms)
            .then(self.end_time_ms.cmp(&other.end_time_ms))
            .reverse()
    }
}

impl PartialOrd for BoundingSpokeTime {
    fn partial_cmp(&self, other: &BoundingSpokeTime) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for BoundingSpokeTime {
    fn eq(&self, other: &BoundingSpokeTime) -> bool {
        self.start_time_ms.eq(&other.start_time_ms) & &self.end_time_ms.eq(&other.end_time_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn can_create_spoke() {
        let s: Spoke = Spoke::new_from_now(10);
        assert_eq!(s.job_list.len(), 0);
    }

    #[test]
    fn can_add_jobs() {
        let current_ms = times::current_time_ms();
        let mut s: Spoke = Spoke::new_from_now(10_000);
        s.add_job(Job::new_auto_id(current_ms + 4000, "Hello Second Job!"));
        assert_eq!(s.job_list.len(), 1);
        s.add_job(Job::new_auto_id(current_ms + 6000, "Hello Second Job!"));
        assert_eq!(s.job_list.len(), 2);
    }

    #[test]
    fn owns_jobs() {
        let current_ms = times::current_time_ms();
        let mut s: Spoke = Spoke::new_from_now(10_000);
        let j = Job::new_auto_id(current_ms + 4000, "Hello Second Job!");
        let id = j.get_metadata().get_id();
        s.add_job(j);
        assert_eq!(
            s.job_list.len(),
            1,
            "Adding a job should increase job list len"
        );
        assert!(s.owns_job(id), "Spoke should advertise ownership of job");

        assert!(
            !s.owns_job(Uuid::new_v4()),
            "Spoke should not advertise ownership of unknown jobs"
        );
    }

    #[test]
    fn walk_empty_spoke() {
        let mut s: Spoke = Spoke::new_from_now(1000);
        let res = s.walk();
        assert_eq!(res.len(), 0, "Empty spoke should have no jobs")
    }

    #[test]
    fn walk_spoke_with_jobs() {
        let current_time = times::current_time_ms();
        let mut s: Spoke = Spoke::new(current_time, 1000);
        s.add_job(Job::new_auto_id(current_time + 300, "I am Job"));
        s.add_job(Job::new_auto_id(current_time + 523, "I am Job"));
        // wait 750 for jobs to be active
        thread::park_timeout(Duration::from_millis(750));
        let res = s.walk();
        assert_eq!(res.len(), 2, "Test should have found 2 jobs ready")
    }

    #[test]
    fn walk_spoke_with_jobs_idempotent() {
        let current_time = times::current_time_ms();
        let mut s: Spoke = Spoke::new(current_time, 10_000);
        println!("Spoke list idempotent: {:p}", &s);

        s.add_job(Job::new_auto_id(current_time + 500, "I am Job"));
        println!("Spoke list idempotent: {:p}", &s);

        s.add_job(Job::new_auto_id(current_time + 500, "I am Job"));
        // wait 3/4 sec
        thread::park_timeout(Duration::from_millis(750));

        let first_job_set = s.walk();
        assert_eq!(
            first_job_set.len(),
            2,
            "Test should have found 2 jobs ready"
        );
        println!("Walk 1 done, pending job len: {:?}", s.pending_job_len());

        let second_job_set = s.walk();
        assert_eq!(
            second_job_set.len(),
            0,
            "Test should have found 0 jobs ready"
        );
        println!("Walk 2 done, pending job len: {:?}", s.pending_job_len());
    }

    #[test]
    fn reject_outoftimebounds_jobs() {
        let current_time = times::current_time_ms();
        // Spoke spanning 20 seconds from now
        let mut s: Spoke = Spoke::new(current_time, 20_000);

        // Accepts jobs that are with Spoke's duration
        let j_accept: Job = Job::new_auto_id(current_time + 7000, "in spoke duration");
        let jj_accept: Job = Job::new_auto_id(current_time + 11_000, "in spoke duration");
        // Rejects jobs that come after Spoke's duration
        let j_reject: Job = Job::new_auto_id(current_time + 44_000, "beyond spoke duration");
        // Rejects jobs that come before Spoke's duration
        let jj_reject: Job = Job::new_auto_id(current_time - 2_000, "before spoke duration");

        assert!(
            s.add_job(j_accept).is_none(),
            "Should accept jobs in spoke span"
        );
        assert!(
            s.add_job(jj_accept).is_none(),
            "Should accept jobs in spoke span"
        );
        assert!(
            s.add_job(j_reject).is_some(),
            "Should reject jobs beyond spoke span"
        );
        assert!(
            s.add_job(jj_reject).is_some(),
            "Should reject jobs before spoke span"
        );
    }

    #[test]
    fn spoke_ordering() {
        let one = Spoke::new_from_now(5);
        thread::park_timeout(Duration::from_millis(10));
        let two = Spoke::new_from_now(5);
        assert!(
            one > two,
            "Spoke with time interval closer to now should be greater"
        );
    }

    #[test]
    fn spoke_from_bounds() {
        let bst = BoundingSpokeTime::new(500, 800);
        let spoke = Spoke::new_from_bounds(bst);

        let bst = BoundingSpokeTime::new(500, 799);
        assert!(spoke.get_bounds().contains(&bst));
    }

    #[test]
    fn can_cancel_job() {
        let current_ms = times::current_time_ms();
        let mut s: Spoke = Spoke::new_from_now(10_000);

        let j_one = Job::new_auto_id(current_ms + 600, "one");
        let j_two = Job::new_auto_id(current_ms + 700, "two");

        let j_one_id = j_one.get_metadata().get_id();
        s.add_job(j_one);
        s.add_job(j_two);

        assert_eq!(s.pending_job_len(), 2);

        // Cancel
        assert!(s.cancel_job(j_one_id));
        // Job is gone, more cancels are idempotent
        assert!(!s.cancel_job(j_one_id));
    }
}
