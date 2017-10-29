// https://serde.rs/derive.html
extern crate bincode;
#[macro_use]
extern crate serde_derive;
extern crate flate2;
extern crate bytes;
extern crate time;

use std::ops::Add;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::DeflateEncoder;
use flate2::read::DeflateDecoder;
use bytes::{BytesMut, BufMut, IntoBuf};
use std::time::{Duration, SystemTime};
use std::collections::BinaryHeap;

pub mod spoke;
pub mod job;

fn main() {

    let mut spokes = spoke::Spoke::new(Duration::new(5, 0));
    let j = job::Job::new(1u64, 1u64, 500u64, "Hello Job!".to_owned());

    spokes.add_job(j);
    spokes.add_job(job::Job::new(
        2u64,
        2u64,
        500u64,
        "Hello Second Job!".to_owned(),
    ));

    //    let serialized = bincode::serialize(&job, bincode::Infinite).unwrap();
    //    println!("serialized = {:?}", serialized);

    //    let e: DeflateEncoder<Vec<u8>> = DeflateEncoder::new(serialized, Compression::Default);
    //    let compressed_bytes = e.finish();
    //    match compressed_bytes {
    //        Ok(b) => {
    //            let mut cb = vec![];
    //            cb.put(b);
    //            let d = DeflateDecoder::new(cb.into_buf()).flush().;
    //            match d {
    //                Ok(foo) => {
    //                    println!("Found foo: {:?}", foo);
    //                }
    //                Err(e) => println!("Error deflating: {:?}", e),
    //            }
    //            //            let mut s = String::new();
    //            //            d.read_to_string(&mut s).unwrap();
    //
    //            let deserialized: job::Job = bincode::deserialize(d).unwrap();
    //            println!("deserialized = {:?}", deserialized);
    //        }
    //        Err(e) => println!("Error compressing: {:?}", e),
    //    };

    //    let deserialized: job::Job = bincode::deserialize(serialized.as_slice()).unwrap();
    //    println!("deserialized = {:?}", deserialized);
    println!("all done!");
    println!("Spoke: {}", spokes);
}
