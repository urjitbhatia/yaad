use std::collections::{BinaryHeap, BTreeMap};
use std::collections::binary_heap::PeekMut;

use times;
use spoke::{Spoke, BoundingSpokeTime};
use job::Job;

const DEFAULT_SPOKE_DURATION_MS: u64 = 10;

#[derive(Debug)]
pub struct Hub {
    bst_heap: BinaryHeap<BoundingSpokeTime>,
    bst_spoke_map: BTreeMap<BoundingSpokeTime, Spoke>,
    past_spoke: Spoke,
}

impl Hub {
    /// Creates a new Hub - a hub orchestrates spokes and jobs. Hub is also responsible for ensuring
    /// that spokes are generated on the fly when a spokeless job is added to the hub.
    ///
    /// A Hub comes with a default `past` spoke which accepts any job whose trigger time is in the
    /// past. The hub will always try to walk this spoke first.
    pub fn new() -> Hub {
        Hub {
            bst_heap: BinaryHeap::new(),
            bst_spoke_map: BTreeMap::new(),
            past_spoke: Spoke::new(0, <u64>::max_value()),
        }
    }

    fn add_spoke(&mut self, spoke: Spoke) {
        println!("Adding a new spoke to hub: {:?}", spoke.get_bounds());
        // TODO: Make these two operations atomic?
        self.bst_heap.push(spoke.get_bounds());
        self.bst_spoke_map.insert(spoke.get_bounds(), spoke);
    }

    /// Walk returns a Vector of Spokes that should be consumed next
    /// Calls to this method can return empty vectors if no spokes are ready yet.
    pub fn walk(&mut self) -> Vec<Spoke> {
        let mut ready_spokes: Vec<Spoke> = vec![];

        while let Some(peeked) = self.bst_heap.peek_mut() {
            if peeked.is_ready() || peeked.is_expired() {
                match self.bst_spoke_map.remove(&PeekMut::pop(peeked)) {
                    Some(s) => ready_spokes.push(s),
                    _ => (),
                };
            } else {
                break;
            }
        }
        ready_spokes
    }

    /// Add a new job to the Hub - the hub will find or create the right spoke for this job
    pub fn add_job(&mut self, job: Job) {
        // If None, past spoke accepted the job, else find the right spoke for it
        println!("\nAdding job to hub. Job trigger: {}", job.trigger_at_ms());
        match self.maybe_add_job_to_past(job) {
            Some(j) => {
                if self.add_job_to_spokes(j).is_some() {
                    panic!("Hub should always accept a job")
                }
            }
            None => (),
        };
    }

    /// Adds a job to the correct spoke based on the Job's trigger time
    fn add_job_to_spokes(&mut self, job: Job) -> Option<Job> {
        let job_bst = Hub::job_bounding_spoke_time(&job);
        match {
            // Try to skip as many bounds as possible : these bounds are before this job's bound
            let next_spoke = self.bst_spoke_map
                .iter_mut()
                .skip_while(|s| s.0 < &job_bst)
                .next();
            // This next spoke is a candidate that might accept this job
            match next_spoke {
                Some(s) => {
                    println!("\nFound potential next spoke");
                    // If spoke exists, try to give it the job
                    match s.1.add_job(job) {
                        // Spoke rejected the job and returned it to us, return to top level match
                        Some(j) => Some(j),
                        // Spoke accepted the job, yay!!
                        None => None,
                    }
                }
                // No spoke found, that we either didn't have a spoke or this job is
                // too far in the future
                None => Some(job),
            }
        } {
            // If we weren't able to assign this job yet, create a spoke that might accept it
            Some(j) => {
                self.add_spoke(Spoke::new_from_bounds(job_bst));
                // Try adding job again, recursively
                self.add_job_to_spokes(j)
            }
            None => None,
        }
    }

    /// Attempts to add a job to the past spoke if the job is in the past and returns None.
    /// Otherwise, returns Some(job)
    fn maybe_add_job_to_past(&mut self, job: Job) -> Option<Job> {
        // If job is old, add to the past spoke
        if job.trigger_at_ms() <= times::current_time_ms() {
            // This job should be handed to the past spoke
            println!("This job is older then current time");
            match self.past_spoke.add_job(job) {
                Some(_) => panic!("Past spoke should always accept a job"),
                None => return None,
            }
        }
        // else, hand it back
        return Option::from(job);
    }

    /// Returns the span of a hypothetical Spoke that should own this job.
    fn job_bounding_spoke_time(job: &Job) -> BoundingSpokeTime {
        let spoke_start = times::floor_ms_from_epoch(job.trigger_at_ms());
        return BoundingSpokeTime::new(spoke_start, spoke_start + DEFAULT_SPOKE_DURATION_MS);
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
        assert_eq!(h.bst_heap.len(), 0);
        assert_eq!(h.bst_spoke_map.len(), 0)
    }

    #[test]
    fn can_add_spokes() {
        let mut h = Hub::new();
        h.add_spoke(Spoke::new(times::current_time_ms(), 10_000));
        assert_eq!(h.bst_heap.len(), 1);
        assert_eq!(h.bst_spoke_map.len(), 1)
    }

    #[test]
    fn walk_empty_hub() {
        let mut h = Hub::new();
        let res = h.walk();
        assert_eq!(h.bst_heap.len(), 0);
        assert_eq!(h.bst_spoke_map.len(), 0);
        assert_eq!(res.len(), 0, "Empty hub walk should return no spokes")
    }

    #[test]
    fn can_add_job_to_past() {
        let mut h = Hub::new();
        h.add_job(Job::new(
            1,
            1,
            times::current_time_ms() - 10_000,
            "I am old",
        ));
        assert_eq!(
            h.past_spoke.pending_job_len(),
            1,
            "Hub should put jobs triggering in the past into it's special, past spoke"
        );
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
        assert_eq!(
            h.bst_heap.len(),
            1,
            "Should list ownership of the newly added spoke"
        );
        assert_eq!(
            h.bst_spoke_map.len(),
            1,
            "Should list ownership of the newly added spoke"
        );

        let walk_one = h.walk();
        assert_eq!(
            walk_one.len(),
            1,
            "Should find a spoke that is ready to be walked"
        );
        assert_eq!(
            h.bst_spoke_map.len(),
            0,
            "Should not own any spokes, it was already consumed"
        );
        assert_eq!(
            h.bst_heap.len(),
            0,
            "Should not own any spokes, it was already consumed"
        );

        // Create another spoke that starts 10ms after the first spoke's starting time
        let second_spoke_start = times::current_time_ms() + 10;
        h.add_spoke(Spoke::new(second_spoke_start, 50));
        assert_eq!(h.bst_heap.len(), 1);
        assert_eq!(h.bst_spoke_map.len(), 1);

        let walk_two = h.walk();
        assert_eq!(
            walk_two.len(),
            0,
            "Hub should not return spokes that are still in the future"
        );

        thread::park_timeout(Duration::from_millis(10));
        assert_eq!(h.bst_spoke_map.len(), 1);
        assert_eq!(h.bst_heap.len(), 1);
        let walk_three = h.walk();
        assert_eq!(
            walk_three.len(),
            1,
            "Hub should now return a spoke that's ready to go"
        );
    }

    #[test]
    fn hub_walk_returns_multiple_ready_spokes() {
        // |
        // |     spoke1                           spoke2         walk1([s2,])
        // | s1<---------5ms--------->s1+5 .2ms. s2(s1+7)<--------5ms--------->s2+50
        // |---------------------------------------------------------------------------------->time
        let mut h = Hub::new();

        let first_spoke_start = times::current_time_ms();
        h.add_spoke(Spoke::new(first_spoke_start, 5));
        assert_eq!(h.bst_spoke_map.len(), 1, "Can add a spoke to a hub");
        assert_eq!(h.bst_heap.len(), 1, "Can add a spoke to a hub");

        let second_spoke_start = first_spoke_start + 5 + 2;
        h.add_spoke(Spoke::new(second_spoke_start, 10));
        assert_eq!(h.bst_spoke_map.len(), 2, "Can add a spoke to a hub");
        assert_eq!(h.bst_heap.len(), 2, "Can add a spoke to a hub");

        thread::park_timeout(Duration::from_millis(10));
        assert_eq!(h.bst_spoke_map.len(), 2);
        assert_eq!(h.bst_heap.len(), 2);
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
        assert!(bst.get_start_time_ms() <= job_trigger_at_ms);
        assert!(job_trigger_at_ms <= bst.get_end_time_ms());
        assert_eq!(
            bst.get_end_time_ms() - bst.get_start_time_ms(),
            super::DEFAULT_SPOKE_DURATION_MS
        );
    }

    #[test]
    fn add_job_to_hub() {
        let start_time_ms = times::current_time_ms();
        let mut hub = Hub::new();
        hub.add_job(Job::new(1, 1, start_time_ms + 2, "one spoke"));
        hub.add_job(Job::new(3, 3, start_time_ms + 3, "one spoke"));
        hub.add_job(Job::new(
            2,
            2,
            start_time_ms + super::DEFAULT_SPOKE_DURATION_MS * 2 + 4,
            "foo",
        ));
        hub.add_job(Job::new(
            4,
            4,
            start_time_ms + super::DEFAULT_SPOKE_DURATION_MS * 2 + 5,
            "foo",
        ));
        assert_eq!(hub.bst_spoke_map.len(), 2);
        // wait for first spoke to become ready
        thread::park_timeout(Duration::from_millis(6));
        println!(
            "Test Diagnostic: current time ms: {}",
            times::current_time_ms()
        );
        let mut walk_one = hub.walk();
        assert_eq!(walk_one.len(), 1);

        let spoke_walk_one = walk_one[0].walk();
        assert_eq!(spoke_walk_one.len(), 2);
    }
}
