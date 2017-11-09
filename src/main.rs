extern crate rand;
extern crate uuid;

#[macro_use]
// our modules
pub mod spoke;
pub mod job;
pub mod hub;
pub mod times;
pub mod demo;

fn main() {
    demo::demo();
}
