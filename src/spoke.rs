use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::collections::{LinkedList, VecDeque};
use std::fmt;
use std::iter;
use std::thread;
use std::cmp::PartialOrd;
use job::{Job, new_job};

/// A Spoke is a time-bound chain of jobs
///
/// Each spoke has a start time and a max Duration (inclusive)
/// Any job that should trigger in this time bound should be handled
/// by this spoke.
pub struct Spoke {
    start_time: SystemTime,
    duration: Duration,
    job_list: LinkedList<Job>,
}

impl Spoke {
    /// Constructs a new Spoke - a time bound chain of jobs
    pub fn new(duration: Duration) -> Spoke {
        let start_time = SystemTime::now();
        let mut job_list = LinkedList::new();

        return Spoke {
            start_time: start_time,
            duration: duration,
            job_list: job_list,
        };
    }

    ///# Example
    ///Create a spoke that starts at 5 sec, 0 ns from now
    ///
    ///```
    ///use Spoke;
    ///let s = Spoke::new(Duration::new(10, 0));
    ///```

    pub fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
    }

    ///Walk returns an iterator that returns jobs in trigger order
    ///
    ///Call walk in a loop like an iterator on this spoke
    pub fn walk(&mut self) -> Vec<&Job> {
        self.job_list
            .iter()
            .take_while(|j| j.trigger_at <= SystemTime::now())
            .collect()
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
        s.add_job(new_job(2u64, 2u64, "Hello Second Job!".to_owned()));
        assert!(s.job_list.len() == 1);
        s.add_job(new_job(1u64, 1u64, "Hello Second Job!".to_owned()));
        assert!(s.job_list.len() == 2);
    }

    #[test]
    fn walk_empty_spoke() {
        let mut s = Spoke::new(Duration::new(1, 0));
        let res = s.walk();
        for j in res {
            assert!(true);
            //            match j {
            //                None =>
            //                Some(_) => panic!("Should not return a job"),
            //            }
        }
    }

    #[test]
    fn walk_spoke_with_jobs() {
        let mut s = Spoke::new(Duration::new(10, 0));
        s.add_job(new_job(1u64, 1u64, "Job with time trigger".to_owned()));
        s.add_job(new_job(1u64, 1u64, "Job with time trigger".to_owned()));
        // wait 3/4 sec
        thread::park_timeout(Duration::from_millis(750));
        let res = s.walk();
        assert!(res.len() == 2, "Test should have found 2 jobs ready");
    }
}
