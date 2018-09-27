use colored::*;
use hub::Hub;
use job::Job;
use rand::{thread_rng, Rng};
use settings;
use statsd::Client;
use std::ops::Index;
use std::sync::{Arc, Mutex};
use std::thread;
use times;

pub fn demo(conf: settings::Settings) {
    println!("Running in demo mode. This will infinitely create a stream of jobs");

    let global_hub = Hub::new(10_000);
    let global_hub_mutex = Mutex::new(global_hub);
    let hub_mutex_rc = Arc::new(global_hub_mutex);
    let hub_producer = Arc::clone(&hub_mutex_rc);
    let hub_consumer = Arc::clone(&hub_mutex_rc);

    let max_jobs = conf.count.unwrap_or(50);

    println!("starting producer thread");
    let producer_thread = thread::Builder::new()
        .name("producer".into())
        .spawn(move || {
            let client = Client::new("127.0.0.1:8125", "yaad.").unwrap();
            println!("{} {}", "Producing total jobs: ".green(), max_jobs);
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
                let log = format!(
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
                println!("{}", log.green());

                let hub_producer_lock = hub_producer.lock();
                match hub_producer_lock {
                    Result::Ok(hpl) => {
                        client.time("demojob.addjob.duration", move || {
                            hpl.add_job(&j);
                        });
                    }
                    _ => (),
                }
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
                        "{}",
                        format!(
                            "Ready job: {:?} to be triggered at: {} current time: {}",
                            j.get_body(),
                            times::to_string(j.trigger_at_ms()),
                            times::to_string(times::current_time_ms())
                        ).blue()
                    );
                    job_counter += 1;
                    println!(
                        "{}",
                        format!("Read total jobs so far: {}", job_counter).blue()
                    );
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
        Result::Ok(_) => println!("{}", "Producer thread finished ok".yellow()),
        Result::Err(e) => println!("{} {:?}", "Producer thread errored".red(), e),
    }
    match consumer_thread.join() {
        Result::Ok(_) => println!("{}", "Consumer thread finished ok".yellow()),
        Result::Err(e) => println!("{} {:?}", "Consumer thread errored".red(), e),
    }
}
