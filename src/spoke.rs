use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::collections::LinkedList;
use std::fmt;
use job::*;

pub struct Spoke {
    start_time: SystemTime,
    duration: Duration,
    job_list: LinkedList<Job>,
}

impl Spoke {
    pub fn new(duration: Duration) -> Spoke {
        let start_time = SystemTime::now();
        let mut job_list = LinkedList::new();

        return Spoke {
            start_time: start_time,
            duration: duration,
            job_list: job_list,
        };
    }

    pub fn add_job(&mut self, job: Job) {
        self.job_list.push_back(job);
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
