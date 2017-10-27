use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::collections::LinkedList;
use std::fmt;
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

    pub fn walk(&mut self) {}
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
}
