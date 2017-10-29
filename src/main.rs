use std::time::SystemTime;

// our modules
pub mod spoke;
pub mod job;
pub mod hub;

fn main() {
    let mut spokes = spoke::Spoke::new(SystemTime::now(), 5000);
    let j = job::Job::new(1u64, 1u64, 500u64, "Hello Job!");

    spokes.add_job(j);
    spokes.add_job(job::Job::new(2u64, 2u64, 500u64, "Hello Second Job!"));

    println!("all done!");
    println!("Spoke: {}", spokes);
}
