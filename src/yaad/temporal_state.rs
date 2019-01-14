//! TemporalState is a classification of time for an object
#[derive(Debug)]
pub enum TemporalState {
    Past,
    Current,
    Future,
}

pub trait Temporal {
    fn as_temporal_state(&self) -> TemporalState;
}
