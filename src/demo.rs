use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};
use std::ops::Index;

use hub::Hub;
use job::Job;
use times;

pub fn demo() {
    println!("Running in demo mode. This will infinitely create a stream of jobs");
    let mut h = Hub::new(10_000);
    let mut r = thread_rng();
    let mut job_counter = 0;
    let job_sample_bodies = vec!["Hello ", "Hey ", "Hi "];

    loop {
        let num = r.next_f32();
        if num < 0.05 {
            // Sometimes, create jobs that are quite some time away in the future
            let time_mult = {
                let mult = r.next_f32();
                if mult < 0.2 {
                    100
                } else if mult < 0.4 {
                    10
                } else {
                    1
                }
            };
            let j = Job::new(
                job_counter,
                job_counter,
                times::current_time_ms() + ((num * 1_000.0) as u64 * time_mult),
                job_sample_bodies.index((r.next_u32() % 3) as usize),
            );
            job_counter += 1;
            println!(
                "\nAdding demo job: {} :: Trigger at {:?} ms from now",
                j.get_body(),
                j.trigger_at_ms() - times::current_time_ms()
            );
            h.add_job(j);
        }

        if job_counter == 10 {
            // Switch into drain mode...
            println!(
                "Added: {} jobs, switching to job drain mode at time: {}",
                job_counter,
                times::current_time_ms()
            );
            while job_counter > 0 {
                h.walk_jobs().iter().for_each(|j| {
                    println!(
                        "Ready job: {} to be triggered at: {} current time: {}",
                        j.get_body(),
                        j.trigger_at_ms(),
                        times::current_time_ms()
                    );
                    job_counter -= 1;
                    println!("Remaining: {} jobs", job_counter);
                    thread::park_timeout(Duration::from_millis(100));
                });
            }
        }

        thread::park_timeout(Duration::from_millis(15));
    }
}
