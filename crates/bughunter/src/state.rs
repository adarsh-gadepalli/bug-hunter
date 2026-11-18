//! Identity of a simulated snapshot. Search will use this later; it does
//! not know what a node or a message is.

pub trait SystemState {
    /// Same snapshot must always hash the same. The explorer uses this
    /// as the identity of a state it has already seen.
    fn fingerprint(&self) -> u64;
}
