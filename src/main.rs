extern crate config;
extern crate rand;
extern crate serde;
extern crate statsd;
extern crate uuid;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

// our modules
pub mod demo;
pub mod hub;
pub mod job;
pub mod settings;
pub mod spoke;
pub mod times;

fn main() {
    let settings = settings::Settings::new();
    match settings {
        Result::Ok(r) => {
            println!("Config parsed OK: {:?}", r);
            match r.mode.as_ref() {
                "demo" => demo::demo(r),
                // not implemented yet
                // "consumer" => demo::consumer(),
                // "producer" => demo::producer(),
                _ => println!("Unknown mode. Exiting..."),
            }
        }
        Result::Err(r) => println!("Error parsing config: {:?}", r),
    }
}
