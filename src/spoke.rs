use std::time::{SystemTime, Duration};
use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;
use std::fmt;

// traits
use std::ops::Add;

// our module
use job::Job;

/// A Spoke is a time-bound chain of jobs
///
/// Each spoke has a start time and a max Duration (inclusive)
/// Any job that should trigger in this time bound should be handled
/// by this spoke.
pub struct Spoke {
    start_time: SystemTime,
    duration: Duration,
    job_list: BinaryHeap<Job>,
}

impl Spoke {
    /// Constructs a new Spoke - a time bound chain of jobs
    ///# Example
    ///Create a spoke that starts at 5 sec, 0 ns from now
    ///
    ///```
    ///use Spoke;
    ///let s = Spoke::new(Duration::new(5, 0));
    ///s.add_job(Job::new(1, 2, 3, "hi");
    ///```
    pub fn new(duration: Duration) -> Spoke {
        let start_time = SystemTime::now();
        let job_list = BinaryHeap::new();

        Spoke {
            start_time,
            duration,
            job_list,
        }
    }

    /// Add a new job into the Spoke
    ///
    /// The spoke can reject a job if it doesn't lie in it's time bounds
    pub fn add_job(&mut self, job: Job) -> Option<Job> {
        if self.start_time <= job.trigger_at() &&
            job.trigger_at() <= self.start_time.add(self.duration) {
            // Only accept jobs that are this spoke's responsibility
            println!("Accepting job");
            self.job_list.push(job);
            return Option::None;
        } else {
            // Return jobs that you don't want to accept
            return Option::from(job);
        }
    }

    ///Walk returns an iterator that returns jobs in trigger order
    ///
    ///Call walk in a loop like an iterator on this spoke
    pub fn walk(&mut self) -> Vec<Job> {
        let mut ready_jobs: Vec<Job> = vec![];

        while let Some(peeked) = self.job_list.peek_mut() {
            if peeked.is_ready() {
                let j = PeekMut::pop(peeked);
                ready_jobs.push(j)
            } else {
                break;
            }
        }
        ready_jobs
    }

    #[inline]
    pub fn pending_job_len(&self) -> usize {
        self.job_list.len()
    }
}

impl fmt::Display for Spoke {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Start time: {:?}, Duration: {:?} sec, NumJobs: {}, JobList: {:?})",
            self.start_time,
            self.duration,
            self.job_list.len(),
            self.job_list
        )
    }
}


#[cfg(test)]
mod tests {
    use super::{Job, Spoke};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn can_create_spoke() {
        let s: Spoke = Spoke::new(Duration::new(10, 0));
        assert_eq!(s.job_list.len(), 0)
    }

    #[test]
    fn can_add_jobs() {
        let mut s: Spoke = Spoke::new(Duration::new(10, 0));
        s.add_job(Job::new(2u64, 2u64, 500u64, "Hello Second Job!"));
        assert_eq!(s.job_list.len(), 1);
        s.add_job(Job::new(1u64, 1u64, 500u64, "Hello Second Job!"));
        assert_eq!(s.job_list.len(), 2)
    }

    #[test]
    fn walk_empty_spoke() {
        let mut s: Spoke = Spoke::new(Duration::new(1, 0));
        let res = s.walk();
        assert_eq!(res.len(), 0, "Empty spoke should have no jobs")
    }

    #[test]
    fn walk_spoke_with_jobs() {
        let mut s: Spoke = Spoke::new(Duration::new(10, 0));
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job"));
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job"));
        // wait 3/4 sec
        thread::park_timeout(Duration::from_millis(750));
        let res = s.walk();
        assert_eq!(res.len(), 2, "Test should have found 2 jobs ready")
    }

    #[test]
    fn walk_spoke_with_jobs_idempotent() {
        let mut s: Spoke = Spoke::new(Duration::new(10, 0));
        println!("Spoke list idempotent: {:p}", &s);
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job"));
        println!("Spoke list idempotent: {:p}", &s);
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job"));
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
        let mut s: Spoke = Spoke::new(Duration::from_millis(2000));

        let j_accept: Job = Job::new_without_external_id(1, 100, "one");
        let jj_accept: Job = Job::new_without_external_id(1, 200, "two");

        let j_reject: Job = Job::new_without_external_id(1, 4400, "three");

        assert!(s.add_job(j_accept).is_none());
        assert!(s.add_job(jj_accept).is_none());
        assert!(s.add_job(j_reject).is_some());
    }
}