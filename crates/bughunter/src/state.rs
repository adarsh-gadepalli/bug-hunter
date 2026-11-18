//! Identity of a simulated snapshot. Search uses this; it is node agnostic.

pub trait SystemState {
    /// fingerprint function - used to identify the state.
    fn fingerprint(&self) -> u64;

    /// observability function - used to emit state for debugging walks
    fn summarize(&self) -> String;
}
