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

    fn add_spoke(&mut self, spoke: Spoke) {
        self.spokes.push(spoke);
    }

    pub fn walk(&mut self) -> Vec<Spoke> {
        let mut ready_spokes: Vec<Spoke> = vec![];
        while let Some(peeked) = self.spokes.peek_mut() {
            if peeked.is_ready() || peeked.is_expired() {
                ready_spokes.push(PeekMut::pop(peeked));
            } else {
                break;
            }
        }
        ready_spokes
    }
}

#[cfg(test)]
mod tests {
    use super::{Spoke, Hub};
    use std::time::{Duration, SystemTime};
    use std::thread;
    use std::ops::Add;

    #[test]
    fn can_create_hub() {
        let h: Hub = Hub::new();
        assert_eq!(h.spokes.len(), 0)
    }

    #[test]
    fn can_add_spokes() {
        let mut h = Hub::new();
        h.add_spoke(Spoke::new(SystemTime::now(), 10_000));
        assert_eq!(h.spokes.len(), 1);
    }

    #[test]
    fn walk_empty_hub() {
        let mut h = Hub::new();
        let res = h.walk();
        assert_eq!(h.spokes.len(), 0);
        assert_eq!(res.len(), 0, "Empty hub walk should return no spokes")
    }

    #[test]
    fn walk_hub_with_spokes() {
        // |
        // |     spoke1  walk1([s1,])            walk2([])         spoke2   walk3([s2,])
        // | s1<---------10ms--------->s1+10 .......~10ms....... s2<--------50ms--------->s2+50
        // |---------------------------------------------------------------------------------->time
        let mut h = Hub::new();
        let first_spoke_start = SystemTime::now();
        h.add_spoke(Spoke::new(first_spoke_start, 10));
        let walk_one = h.walk();
        assert_eq!(walk_one.len(), 1, "Should find a spoke that is ready to be walked");

        let second_spoke_start = SystemTime::now().add(Duration::from_millis(10));
        h.add_spoke(Spoke::new(second_spoke_start, 50));
        assert_eq!(h.spokes.len(), 1);
        let walk_two = h.walk();
        assert_eq!(walk_two.len(), 0, "Hub should not return spokes that are still in the future");

        thread::park_timeout(Duration::from_millis(10));
        assert_eq!(h.spokes.len(), 1);
        let walk_three = h.walk();
        assert_eq!(walk_three.len(), 1, "Hub should now return a spoke that's ready to go");
    }

    #[test]
    fn hub_walk_returns_multiple_ready_jobs() {
        // |
        // |     spoke1                           spoke2         walk1([s2,])
        // | s1<---------5ms--------->s1+5 .2ms. s2(s1+7)<--------5ms--------->s2+50
        // |---------------------------------------------------------------------------------->time
        let mut h = Hub::new();

        let first_spoke_start = SystemTime::now();
        h.add_spoke(Spoke::new(first_spoke_start, 5));
        assert_eq!(h.spokes.len(), 1, "Can add a spoke to a hub");

        let second_spoke_start = first_spoke_start.add(Duration::from_millis(2 + 5));
        h.add_spoke(Spoke::new(second_spoke_start, 10));
        assert_eq!(h.spokes.len(), 2, "Can add a spoke to a hub");

        thread::park_timeout(Duration::from_millis(10));
        assert_eq!(h.spokes.len(), 2);
        let walk_one = h.walk();
        assert_eq!(walk_one.len(), 2, "Hub should now both spokes that are ready to go");
        assert!(walk_one[0] > walk_one[1], "Hub returns spokes in order of closest to first");
    }
}