/*
The "Job" type - max possible values: u64::max_value() = 18446744073709551615.
internal_id will overflow after max value - internal functioning should not be affected.
*/
//#[derive(Serialize, Deserialize, Debug)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Job {
    internal_id: u64,
    external_id: u64,
    body: String,
}

pub fn new_job(internal_id: u64, external_id: u64, body: String) -> Job {
    return Job {
        internal_id: internal_id,
        external_id: external_id,
        body: body,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_create_job() {
        let j = new_job(100u64, 150u64, "Test Body".to_owned());
        assert!(j.internal_id == 100u64)
    }
}
