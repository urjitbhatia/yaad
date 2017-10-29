use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;

use spoke::Spoke;

pub struct Hub {
    spokes: BinaryHeap<Spoke>,
}

impl Hub {
    pub fn new() -> Hub {
        Hub { spokes: BinaryHeap::new() }
    }

    pub fn walk(&mut self) {
        while let Some(peeked) = self.spokes.peek_mut() {
            if peeked.is_ready() && peeked.pending_job_len() > 0 {
                PeekMut::pop(peeked);
            }
        }
    }
}