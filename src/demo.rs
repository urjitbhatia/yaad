use rand::{thread_rng, Rng};
use std::ops::Index;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use hub::Hub;
use job::Job;
use settings;
use statsd::Client;
use times;

pub fn demo(conf: settings::Settings) {
    println!("Running in demo mode. This will infinitely create a stream of jobs");
    let client = Client::new("127.0.0.1:8125", "yaad.").unwrap();

    // let mut h = Hub::new(10_000);
    let hub_mutex_rc = Arc::new(Mutex::new(Hub::new(10_000)));
    let hub_producer = Arc::clone(&hub_mutex_rc);
    let hub_consumer = Arc::clone(&hub_mutex_rc);

    let max_jobs = conf.count.unwrap_or(50);

    let producer_thread = thread::Builder::new()
        .name("producer".into())
        .spawn(move || {
            let mut h = hub_producer.lock().unwrap();
            let job_sample_bodies = vec!["Hello ", "Hey ", "Hi "];
            let mut r = thread_rng();
            println!("producing {:?} jobs", max_jobs);
            for job_counter in 0..max_jobs {
                let num = r.next_f32();
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
                let j = Job::new_auto_id(
                    times::current_time_ms() + ((num * 1_000.0) as u64 * time_mult),
                    job_sample_bodies.index((r.next_u32() % 3) as usize),
                );
                client.incr("demojob.produced");
                let trigger_at_from_now =
                    j.trigger_at_ms() as i64 - times::current_time_ms() as i64;
                println!(
                    "\nAdding demo job: {:?} :: Trigger at {:?} ms \
                         Current time: {:?}. 
                        Trigger at {:?}ms from now
                         Count: {}",
                    j.get_body(),
                    j.trigger_at_ms(),
                    times::current_time_ms(),
                    trigger_at_from_now,
                    job_counter
                );
                if trigger_at_from_now.lt(&0) {
                    println!("******* Trigger is past: {:?}", j);
                }
                h.add_job(j);
            }
        }).unwrap();

    let consumer_thread = thread::Builder::new()
        .name("consumer".into())
        .spawn(move || {
            let mut h = hub_consumer.lock().unwrap();
            println!("-----------------------------------------------");
            // Switch into drain mode...
            println!("Job drain mode",);
            println!("-----------------------------------------------");
            let mut job_counter = 0;
            loop {
                h.walk_jobs().iter().for_each(|j| {
                    println!(
                        "Ready job: {:?} to be triggered at: {} current time: {}",
                        j.get_body(),
                        j.trigger_at_ms(),
                        times::current_time_ms()
                    );
                    job_counter += 1;
                    println!("Processed: {} jobs", job_counter);
                });

                if job_counter.eq(&max_jobs) {
                    println!("Consumer done with max jobs");
                    break;
                }
                thread::park_timeout(Duration::from_millis(15));
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
