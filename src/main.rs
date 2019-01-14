extern crate chrono;
extern crate colored;
extern crate config;
extern crate rand;
extern crate serde;
extern crate statsd;
extern crate uuid;

extern crate tokio;
#[macro_use]
extern crate futures;
extern crate bytes;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;

// our modules
mod demo;
mod protocols;
mod settings;
mod yaad;

use std::error::Error;

extern crate pretty_env_logger;

fn main() {
    pretty_env_logger::init_timed();

    let settings = settings::Settings::new();
    match settings {
        Result::Ok(r) => {
            info!("Config parsed OK: {:?}", r);
            match r.mode.as_ref() {
                "demo" => demo::demo(r),
                "beanstalkd" => protocols::beanstalkd::run(r),
                _ => info!("Unknown mode. Exiting..."),
            }
        }
        Result::Err(r) => {
            info!("Error parsing config: {:?}", r);
            info!("Error source: {:?}", r.source());
        }
    }
}
