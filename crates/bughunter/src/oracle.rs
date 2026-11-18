//! Invariants checked on every state the walk reaches, and the report a violation produces.


// the oracle checks the state of the system for any violating states.
pub trait Oracle<S> {
    fn name(&self) -> &'static str;

    /// `Some(reason)` if `state` breaks the invariant.
    fn check(&self, state: &S) -> Option<String>;
}

// a struct containing the oracle name, the reason for the violation, and the step number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BugReport {
    /// the name of the oracle that found the violation.
    pub oracle: String,
    /// the reason for the violation.
    pub reason: String,
    /// the step number at which the violation was found.
    pub step: usize,
}
