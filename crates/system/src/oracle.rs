//! The invariant every explored snapshot must satisfy.

use bughunter::{Oracle, SystemState};

use crate::system::DummySystem;

/// Once no messages are in flight, every node must hold the same value.
pub struct Converged;

impl Oracle<DummySystem> for Converged {
    
    // the name of the oracle.
    fn name(&self) -> &'static str {
        "converged"
    }

    fn check(&self, state: &DummySystem) -> Option<String> {
        // checks that all nodes have an empty inbox.
        let quiet = state.nodes.iter().all(|node| node.inbox.is_empty());

        // checks if all nodes have the same local data.
        let first = state.nodes[0].local;
        let disagree = state.nodes.iter().any(|node| node.local != first);
        
        // if the system is quiet and the nodes disagree, return a formatted string with the state summary.
        if quiet && disagree {
            Some(format!(
                "nothing in flight but nodes disagree: {}",
                state.summarize()
            ))
        } else {
            // else, we good
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_while_messages_are_in_flight() {
        let mut state = DummySystem::initial();
        state.nodes[0].local = 2;
        state.nodes[1].inbox.push(crate::Message {
            from: 0,
            payload: 2,
        });
        assert_eq!(Converged.check(&state), None);
    }

    #[test]
    fn fails_when_quiet_and_values_differ() {
        let mut state = DummySystem::initial();
        state.nodes[2].local = 1;
        assert!(Converged.check(&state).is_some());
    }
}
