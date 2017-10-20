/*
The "Job" type - max possible values: u64::max_value() = 18446744073709551615.
internal_id will overflow after max value - internal functioning should not be affected.
*/

//#[derive(Serialize, Deserialize, Debug)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Job<'job> {
    internal_id: u64,
    external_id: u64,
    body: &'job str,
}

pub fn new_job(internal_id: u64, external_id: u64, body: &str) -> Job {
    return Job {
        internal_id: internal_id,
        external_id: external_id,
        body: body,
    };
}
