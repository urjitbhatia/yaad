use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::collections::{LinkedList, VecDeque};
use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;
use std::fmt;
use std::iter;
use std::thread;
use std::cmp::PartialOrd;
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
    pub fn new(duration: Duration) -> Spoke {
        let start_time = SystemTime::now();
        let job_list = BinaryHeap::new();

        println!("Job list ref: {:p}", &job_list);

        Spoke { start_time, duration, job_list }
    }

    ///# Example
    ///Create a spoke that starts at 5 sec, 0 ns from now
    ///
    ///```
    ///use Spoke;
    ///let s = Spoke::new(Duration::new(10, 0));
    ///```

    pub fn add_job(&mut self, job: Job) {
        println!("Job list ref add_job: {:p}", &self.job_list);
        self.job_list.push(job);
    }

    ///Walk returns an iterator that returns jobs in trigger order
    ///
    ///Call walk in a loop like an iterator on this spoke
    pub fn walk(&mut self) -> Vec<Job> {
        let mut ready_jobs: Vec<Job> = vec![];

        while let Some(peeked) = self.job_list.peek_mut() {
            if peeked.trigger_at <= SystemTime::now() {
                let j = PeekMut::pop(peeked);
                ready_jobs.push(j)
            }
        }
        ready_jobs
    }

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
    use super::*;

    #[test]
    fn can_create_spoke() {
        let s = Spoke::new(Duration::new(10, 0));
        assert!(s.job_list.len() == 0);
    }

    #[test]
    fn can_add_jobs() {
        let mut s = Spoke::new(Duration::new(10, 0));
        s.add_job(Job::new(2u64, 2u64, 500u64, "Hello Second Job!".to_owned()));
        assert!(s.job_list.len() == 1);
        s.add_job(Job::new(1u64, 1u64, 500u64, "Hello Second Job!".to_owned()));
        assert!(s.job_list.len() == 2);
    }

    #[test]
    fn walk_empty_spoke() {
        let mut s = Spoke::new(Duration::new(1, 0));
        let res = s.walk();
        for j in res {
            panic!("Empty spoke should have no jobs");
        }
    }

    #[test]
    fn walk_spoke_with_jobs() {
        let mut s = Spoke::new(Duration::new(10, 0));
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job".to_owned()));
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job".to_owned()));
        // wait 3/4 sec
        thread::park_timeout(Duration::from_millis(750));
        let res = s.walk();
        assert!(res.len() == 2, "Test should have found 2 jobs ready");
    }

    #[test]
    fn walk_spoke_with_jobs_idempotent() {
        let mut s = Spoke::new(Duration::new(10, 0));
        println!("Spoke list idempotent: {:p}", &s);
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job".to_owned()));
        println!("Spoke list idempotent: {:p}", &s);
        s.add_job(Job::new(1u64, 1u64, 500u64, "I am Job".to_owned()));
        // wait 3/4 sec
        thread::park_timeout(Duration::from_millis(750));
        let first_job_set = s.walk();
        assert!(
            first_job_set.len() == 2,
            "Test should have found 2 jobs ready"
        );
        println!("Walk 1 done, pending job len: {:?}", s.pending_job_len());

        let second_job_set = s.walk();
        assert!(
            second_job_set.len() == 0,
            "Test should have found 0 jobs ready"
        );
        println!("Walk 2 done, pending job len: {:?}", s.pending_job_len());
    }
}
