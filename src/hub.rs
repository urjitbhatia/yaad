use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;

use times;
use spoke::Spoke;
use job::Job;

const DEFAULT_SPOKE_DURATION_MS: u64 = 10;

pub struct Hub {
    spokes: BinaryHeap<Spoke>,
}

#[derive(Debug)]
pub struct BoundingSpokeTime {
    start_ms: u64,
    end_ms: u64,
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

    #[allow(unused_variables)]
    pub fn add_job(&mut self, job: Job) {
        unimplemented!()
    }

    /// Returns the span of a hypothetical Spoke that should own this job.
    pub fn job_bounding_spoke_time(job: &Job) -> BoundingSpokeTime {
        let spoke_start = times::floor_ms_from_epoch(job.trigger_at_ms());
        return BoundingSpokeTime {
            start_ms: spoke_start,
            end_ms: spoke_start + DEFAULT_SPOKE_DURATION_MS,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{Spoke, Hub, Job, times};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use std::thread;

    #[test]
    fn can_create_hub() {
        let h: Hub = Hub::new();
        assert_eq!(h.spokes.len(), 0)
    }

    #[test]
    fn can_add_spokes() {
        let mut h = Hub::new();
        h.add_spoke(Spoke::new(times::current_time_ms(), 10_000));
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
        let first_spoke_start = times::current_time_ms();

        // Create a spoke that starts now and add it to the hub
        h.add_spoke(Spoke::new(first_spoke_start, 10));
        assert_eq!(h.spokes.len(), 1, "Should list ownership of the newly added spoke");

        let walk_one = h.walk();
        assert_eq!(
            walk_one.len(),
            1,
            "Should find a spoke that is ready to be walked"
        );
        assert_eq!(h.spokes.len(), 0, "Should not own any spokes, it was already consumed");

        // Create another spoke that starts 10ms after the first spoke's starting time
        let second_spoke_start = times::current_time_ms() + 10;
        h.add_spoke(Spoke::new(second_spoke_start, 50));
        assert_eq!(h.spokes.len(), 1);

        let walk_two = h.walk();
        assert_eq!(
            walk_two.len(),
            0,
            "Hub should not return spokes that are still in the future"
        );

        thread::park_timeout(Duration::from_millis(10));
        assert_eq!(h.spokes.len(), 1);
        let walk_three = h.walk();
        assert_eq!(
            walk_three.len(),
            1,
            "Hub should now return a spoke that's ready to go"
        );
    }

    #[test]
    fn hub_walk_returns_multiple_ready_jobs() {
        // |
        // |     spoke1                           spoke2         walk1([s2,])
        // | s1<---------5ms--------->s1+5 .2ms. s2(s1+7)<--------5ms--------->s2+50
        // |---------------------------------------------------------------------------------->time
        let mut h = Hub::new();

        let first_spoke_start = times::current_time_ms();
        h.add_spoke(Spoke::new(first_spoke_start, 5));
        assert_eq!(h.spokes.len(), 1, "Can add a spoke to a hub");

        let second_spoke_start = first_spoke_start + 5 + 2;
        h.add_spoke(Spoke::new(second_spoke_start, 10));
        assert_eq!(h.spokes.len(), 2, "Can add a spoke to a hub");

        thread::park_timeout(Duration::from_millis(10));
        assert_eq!(h.spokes.len(), 2);
        let walk_one = h.walk();
        assert_eq!(
            walk_one.len(),
            2,
            "Hub should now both spokes that are ready to go"
        );
        assert!(
            walk_one[0] > walk_one[1],
            "Hub returns spokes in order of closest to first"
        );
    }

    /// This test checks that we can calculate if a Spoke should own a job - a spoke should own a
    /// job if that job's trigger time lies within the Spoke's duration.
    #[test]
    fn test_job_bounding_spoke_times() {
        // Find Duration since UNIX_EPOCH
        let dur_from_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        // Find the ms since EPOCH, floored to the nearest decimal
        let ms_from_epoch = times::floor_ms_from_epoch(times::duration_to_ms(dur_from_epoch));

        // ms from epoch down to closest 10 and then add 10ms
        let ms_from_epoch = ms_from_epoch + 10;
        let job_trigger_at_ms = ms_from_epoch + 7;
        let j = Job::new(1, 1, job_trigger_at_ms, "foo");

        // This job's bounds should be: ms_from_epoch -> ms_from_epoch + 10
        let bst = self::Hub::job_bounding_spoke_time(&j);
        assert!(bst.start_ms <= job_trigger_at_ms);
        assert!(job_trigger_at_ms <= bst.end_ms);
        assert_eq!(bst.end_ms - bst.start_ms, super::DEFAULT_SPOKE_DURATION_MS);
    }
}
