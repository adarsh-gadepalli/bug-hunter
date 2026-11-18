//! The system under test, as the explorer sees it: a start state, the legal
//! next actions, and a pure function that applies one.

use std::fmt::Debug;

use crate::SystemState;

pub trait SystemModel {

    /// state type - must implement SystemState and be cloneable.
    type State: SystemState + Clone;

    /// action type - must be cloneable and debuggable.
    type Action: Clone + Debug;

    /// deterministic initial state function
    fn initial_state(&self) -> Self::State;

    /// deterministic action retrieval function
    fn get_actions(&self, state: &Self::State) -> Vec<Self::Action>;

    /// deterministic transition function
    fn transition(&self, state: &Self::State, action: &Self::Action) -> Self::State;
}
