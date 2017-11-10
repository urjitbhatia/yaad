use std::collections::{BTreeMap};

use times;
use spoke::{Spoke, BoundingSpokeTime};
use job::Job;
use uuid::Uuid;

#[derive(Debug)]
pub struct Hub {
    spoke_duration_ms: u64,
    bst_spoke_map: BTreeMap<BoundingSpokeTime, Spoke>,
    past_spoke: Spoke,
}

impl Hub {
    /// Creates a new Hub - a hub orchestrates spokes and jobs. Hub is also responsible for ensuring
    /// that spokes are generated on the fly when a spokeless job is added to the hub.
    ///
    /// A Hub comes with a default `past` spoke which accepts any job whose trigger time is in the
    /// past. The hub will always try to walk this spoke first.
    pub fn new(spoke_duration_ms: u64) -> Hub {
        Hub {
            spoke_duration_ms,
            bst_spoke_map: BTreeMap::new(),
            past_spoke: Spoke::new(0, <u64>::max_value()),
        }
    }

    pub fn find_job_owner_bst(&self, id: Uuid) -> Option<BoundingSpokeTime> {
        if self.past_spoke.owns_job(id) {
            return Some(self.past_spoke.get_bounds().clone());
        }
        let entry = self.bst_spoke_map
            .iter()
            .skip_while(|e| !e.1.owns_job(id))
            .next();
        if entry.is_some() {
            return Some(entry.unwrap().0.clone());
        }
        None
    }

    fn add_spoke(&mut self, spoke: Spoke) {
        self.bst_spoke_map.insert(spoke.get_bounds(), spoke);
    }

    /// Walk returns a Vector of Spokes that should be consumed next
    /// Calls to this method can return empty vectors if no spokes are ready yet.
    pub fn walk(&mut self) -> Vec<Job> {
        let mut ready_jobs: Vec<Job> = vec![];
        self.bst_spoke_map
            .values_mut()
            .take_while(|s| s.is_ready())
            .for_each(|s| ready_jobs.append(s.walk().as_mut()));
        self.prune_spokes();
        ready_jobs
    }

    pub fn prune_spokes(&mut self) -> u32 {
        let to_remove: Vec<BoundingSpokeTime> = self.bst_spoke_map
            .values()
            .take_while(|s| s.is_expired() && s.pending_job_len() == 0)
            .map(|s| s.get_bounds())
            .collect();
        let mut prune_count = 0;
        for k in to_remove {
            if self.bst_spoke_map.remove(&k).is_some() {
                prune_count += 1;
            }
        }
        return prune_count;
    }

    /// Add a new job to the Hub - the hub will find or create the right spoke for this job
    pub fn add_job(&mut self, job: Job) -> &mut Hub {
        // If None, past spoke accepted the job, else find the right spoke for it
        println!("Adding job to hub. Job trigger: {}", job.trigger_at_ms());
        match self.maybe_add_job_to_past(job) {
            Some(j) => {
                if self.add_job_to_spokes(j).is_some() {
                    panic!("Hub should always accept a job")
                }
                return self;
            }
            None => return self,
        }
    }

    /// Adds a job to the correct spoke based on the Job's trigger time
    fn add_job_to_spokes(&mut self, job: Job) -> Option<Job> {
        let job_bst = Hub::job_bounding_spoke_time(&job, self.spoke_duration_ms);
        match {
            // Try to skip as many bounds as possible : these bounds are before this job's bound
            let next_spoke = self.bst_spoke_map
                .iter_mut()
                .skip_while(|s| s.0 < &job_bst)
                .next();
            // This next spoke is a candidate that might accept this job
            match next_spoke {
                Some(s) => {
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
                println!("Adding a new spoke to accomodate job: {:?}", job_bst);
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
        let current_time_ms = times::current_time_ms();
        if job.trigger_at_ms() < current_time_ms {
            // This job should be handed to the past spoke
            println!(
                "This job: {} is older then current time: {}",
                job.trigger_at_ms(),
                current_time_ms
            );
            match self.past_spoke.add_job(job) {
                Some(_) => panic!("Past spoke should always accept a job"),
                None => return None,
            }
        }
        // else, hand it back
        return Option::from(job);
    }

    /// Returns the span of a hypothetical Spoke that should own this job.
    fn job_bounding_spoke_time(job: &Job, spoke_duration_ms: u64) -> BoundingSpokeTime {
        let spoke_start = times::floor_ms_from_epoch(job.trigger_at_ms());
        return BoundingSpokeTime::new(spoke_start, spoke_start + spoke_duration_ms);
    }

    /// Returns a vec of all jobs that are ready to be consumed
    pub fn walk_jobs(&mut self) -> Vec<Job> {
        let mut jobs = self.past_spoke.walk();
        jobs.append(self.walk().as_mut());
        return jobs;
    }
}


#[cfg(test)]
mod tests {
    const TEST_SPOKE_DURATION_MS: u64 = 10;

    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use std::thread;

    #[test]
    fn can_create_hub() {
        let h: Hub = Hub::new(TEST_SPOKE_DURATION_MS);
        assert_eq!(h.bst_spoke_map.len(), 0)
    }

    #[test]
    fn can_add_spokes() {
        let mut h = Hub::new(TEST_SPOKE_DURATION_MS);
        h.add_spoke(Spoke::new(times::current_time_ms(), 10_000));
        assert_eq!(h.bst_spoke_map.len(), 1)
    }

    #[test]
    fn walk_empty_hub() {
        let mut h = Hub::new(TEST_SPOKE_DURATION_MS);
        let res = h.walk();
        assert_eq!(h.bst_spoke_map.len(), 0);
        assert_eq!(res.len(), 0, "Empty hub walk should return no spokes")
    }

    #[test]
    fn can_add_job_to_past() {
        let mut h = Hub::new(TEST_SPOKE_DURATION_MS);
        h.add_job(Job::new_auto_id(
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
        // | s1<---------20ms--------->s1+10 .......~10ms....... s2<--------25ms--------->s2+50
        // |---------------------------------------------------------------------------------->time
        let mut h = Hub::new(TEST_SPOKE_DURATION_MS);
        let first_spoke_start = times::current_time_ms();
        // Create a spoke that starts now and add it to the hub
        let mut s1 = Spoke::new(first_spoke_start, 10);
        s1.add_job(Job::new_auto_id(first_spoke_start + 2, "job"));
        h.add_spoke(s1);

        assert_eq!(
            h.bst_spoke_map.len(),
            1,
            "Should list ownership of the newly added spoke"
        );

        // Wait for at least first spoke to be ready
        thread::park_timeout(Duration::from_millis(15));

        assert_eq!(
            h.walk_jobs().len(),
            1,
            "Should have 1 job ready at: {}",
            times::current_time_ms()
        );
        assert_eq!(
            h.bst_spoke_map.len(),
            0,
            "Should have pruned spoke after consuming it"
        );


        // Create another spoke that starts 10ms after the first spoke's starting time
        let second_spoke_start = times::current_time_ms() + 10;
        let mut s2 = Spoke::new(second_spoke_start, 25);
        s2.add_job(Job::new_auto_id(second_spoke_start + 17, "job"));
        h.add_spoke(s2);

        assert_eq!(h.bst_spoke_map.len(), 1, "Should have 1 spoke");

        // Wait for at least second spoke to be ready
        thread::park_timeout(Duration::from_millis(30));
        assert_eq!(h.walk_jobs().len(), 1, "Hub should return jobs");

        assert_eq!(
            h.walk_jobs().len(),
            0,
            "Hub should not return a job after all were consumed"
        );

        thread::park_timeout(Duration::from_millis(10));
        h.prune_spokes();
        assert_eq!(h.bst_spoke_map.len(), 0);
    }

    #[test]
    fn prune_expired_spokes() {
        // |
        // |     spoke1                           spoke2         walk1([s2,])
        // | s1<---------5ms--------->s1+5 .2ms. s2(s1+7)<--------5ms--------->s2+50
        // |---------------------------------------------------------------------------------->time
        let mut h = Hub::new(TEST_SPOKE_DURATION_MS);

        let first_spoke_start = times::current_time_ms();
        h.add_spoke(Spoke::new(first_spoke_start, TEST_SPOKE_DURATION_MS));
        assert_eq!(h.bst_spoke_map.len(), 1, "Can add a spoke to a hub");

        let second_spoke_start = first_spoke_start + TEST_SPOKE_DURATION_MS + 2;
        h.add_spoke(Spoke::new(second_spoke_start, 10));
        assert_eq!(h.bst_spoke_map.len(), 2, "Can add a spoke to a hub");

        thread::park_timeout(Duration::from_millis(TEST_SPOKE_DURATION_MS * 2 + 5));
        assert_eq!(h.bst_spoke_map.len(), 2);
        assert_eq!(h.prune_spokes(), 2, "Expired spokes are pruned");
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
        let j = Job::new_auto_id(job_trigger_at_ms, "foo");

        // This job's bounds should be: ms_from_epoch -> ms_from_epoch + 10
        let bst = self::Hub::job_bounding_spoke_time(&j, TEST_SPOKE_DURATION_MS);
        assert!(bst.get_start_time_ms() <= job_trigger_at_ms);
        assert!(job_trigger_at_ms <= bst.get_end_time_ms());
        assert_eq!(
            bst.get_end_time_ms() - bst.get_start_time_ms(),
            TEST_SPOKE_DURATION_MS
        );
    }

    #[test]
    fn add_job_to_hub() {
        let start_time_ms = times::current_time_ms();
        println!("-- Test Diagnostic: current_time_ms: {}\n", start_time_ms);
        let mut hub = Hub::new(TEST_SPOKE_DURATION_MS);
        // first spoke
        hub.add_job(Job::new_auto_id(start_time_ms + 3, "one spoke"))
            .add_job(Job::new_auto_id(start_time_ms + 4, "one spoke"));

        // next spoke
        hub.add_job(Job::new_auto_id(
            start_time_ms + TEST_SPOKE_DURATION_MS * 2 + 4,
            "foo",
        )).add_job(Job::new_auto_id(
                start_time_ms + TEST_SPOKE_DURATION_MS * 2 + 3,
                "foo",
            ));

        assert_eq!(
            hub.bst_spoke_map.len(),
            2,
            "Failed at time: {}",
            times::current_time_ms()
        );
        // wait for first spoke to become ready
        thread::park_timeout(Duration::from_millis(TEST_SPOKE_DURATION_MS + 2));

        println!(
            "Test Diagnostic: current time ms: {}",
            times::current_time_ms()
        );

        let mut walk_one = hub.walk_jobs();
        assert_eq!(
            walk_one.len(),
            2,
            "Failed at time: {}",
            times::current_time_ms()
        );
    }

    #[test]
    fn can_find_jobs() {
        let start_time_ms = times::current_time_ms();
        let mut hub = Hub::new(TEST_SPOKE_DURATION_MS);
        let job_one_spoke = Job::new_auto_id(start_time_ms + 3, "one spoke");
        let job_other_spoke =
            Job::new_auto_id(start_time_ms + TEST_SPOKE_DURATION_MS * 2 + 4, "foo");

        let job_one_id = job_one_spoke.get_metadata().get_id();
        let job_other_id = job_other_spoke.get_metadata().get_id();

        hub.add_job(job_one_spoke)
            .add_job(Job::new_auto_id(start_time_ms + 4, "one spoke"))
            .add_job(job_other_spoke)
            .add_job(Job::new_auto_id(
                start_time_ms + TEST_SPOKE_DURATION_MS * 2 + 3,
                "foo",
            ));
        assert_eq!(hub.bst_spoke_map.len(), 2);

        assert!(hub.find_job_owner_bst(job_one_id).is_some());
        assert!(hub.find_job_owner_bst(job_other_id).is_some());

        // Can't find unknown jobs
        assert!(!hub.find_job_owner_bst(Uuid::new_v4()).is_some());
    }

    #[test]
    fn can_find_past_jobs() {
        let start_time_ms = times::current_time_ms();
        let mut hub = Hub::new(TEST_SPOKE_DURATION_MS);
        let j = Job::new_auto_id(start_time_ms - 300, "one spoke");

        let id = j.get_metadata().get_id();

        hub.add_job(j);
        assert_eq!(hub.bst_spoke_map.len(), 0);
        assert!(hub.find_job_owner_bst(id).is_some());
        // Is Idempotent
        assert!(hub.find_job_owner_bst(id).is_some());
    }
}
