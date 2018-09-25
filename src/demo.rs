use rand::{thread_rng, Rng};
use std::cell::Cell;
use std::ops::Index;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use hub::Hub;
use job::Job;
use settings;
use statsd::Client;
use times;

pub fn demo(conf: settings::Settings) {
    println!("Running in demo mode. This will infinitely create a stream of jobs");

    let hub_mutex_rc = Arc::new(Mutex::new(Hub::new(10_000)));
    let hub_producer = Arc::clone(&hub_mutex_rc);
    let hub_consumer = Arc::clone(&hub_mutex_rc);

    let max_jobs = conf.count.unwrap_or(50);

    println!("starting producer thread");
    let producer_thread = thread::Builder::new()
        .name("producer".into())
        .spawn(move || {
            let client = Client::new("127.0.0.1:8125", "yaad.").unwrap();
            println!("producing {:?} jobs", max_jobs);
            let job_sample_bodies = vec!["Hello ", "Hey ", "Hi "];
            let mut r = thread_rng();
            for job_counter in 0..max_jobs {
                let delay = match Some(r.next_f32()) {
                    Some(x) if x <= 0.1 => x * -10.0,
                    Some(x) if x <= 0.3 => x * 100.0,
                    Some(x) if x <= 0.5 => x * 10.0,
                    Some(x) => x,
                    _ => 1.0,
                } as u64
                    * 1_000;
                let mut j = Job::new_auto_id(
                    times::current_time_ms() + delay,
                    job_sample_bodies.index((r.next_u32() % 3) as usize),
                );
                client.incr("demojob.produced.count");
                let trigger_at_from_now =
                    j.trigger_at_ms() as i64 - times::current_time_ms() as i64;
                println!(
                    "\nAdding demo job: {:?} :: \
                     Trigger at {:?} ms Current time: {:?}.  \
                     Trigger at {:?} ms from now \
                     Count: {}",
                    j.get_body(),
                    times::to_string(j.trigger_at_ms()),
                    times::to_string(times::current_time_ms()),
                    trigger_at_from_now,
                    job_counter
                );
                let p = Arc::clone(&hub_mutex_rc);
                let mut hp = p.lock().unwrap();
                client.time("demojob.addjob.duration", || {
                    hp.add_job(j);
                });
            }
        }).unwrap();

    let consumer_thread = thread::Builder::new()
        .name("consumer".into())
        .spawn(move || {
            let client = Client::new("127.0.0.1:8125", "yaad.").unwrap();
            println!("-----------------------------------------------");
            // Switch into drain mode...
            println!("Job drain mode",);
            println!("-----------------------------------------------");
            let mut job_counter = 0;
            loop {
                let mut h = hub_consumer.lock().unwrap();

                h.walk_jobs().iter().for_each(|j| {
                    println!(
                        "Ready job: {:?} to be triggered at: {} current time: {}",
                        j.get_body(),
                        times::to_string(j.trigger_at_ms()),
                        times::to_string(times::current_time_ms())
                    );
                    job_counter += 1;
                    println!("Processed: {} jobs", job_counter);
                    client.incr("demojob.consumed.count");
                });

                if job_counter.eq(&max_jobs) {
                    println!("Consumer done with max jobs");
                    break;
                }
                // thread::sleep(time::Duration::from_millis(100));
                thread::yield_now();
            }
        }).unwrap();

    match producer_thread.join() {
        Result::Ok(_) => println!("Producer thread finished ok"),
        Result::Err(e) => println!("Producer thread errored {:?}", e),
    }
    match consumer_thread.join() {
        Result::Ok(_) => println!("Consumer thread finished ok"),
        Result::Err(e) => println!("Consumer thread errored {:?}", e),
    }
}
