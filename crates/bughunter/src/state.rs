//! Identity of a simulated snapshot. Search will use this later; it is node agnostic.

pub trait SystemState {

    /// Placeholder function.
    fn fingerprint(&self) -> u64;
}
