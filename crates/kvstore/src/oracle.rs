//! The invariant every explored snapshot must satisfy.

use bughunter::{Oracle, SystemState};

use crate::store::{KEYS, KvCluster};

/// Once no updates are in flight, every node must hold the same value for every key.
pub struct Converged;

impl Oracle<KvCluster> for Converged {
    fn name(&self) -> &'static str {
        "converged"
    }

    fn check(&self, state: &KvCluster) -> Option<String> {
        let quiet = state.nodes.iter().all(|node| node.inbox.is_empty());
        if !quiet {
            return None;
        }
        // report the first key the nodes disagree on.
        let key = (0..KEYS).find(|&key| {
            let first = state.nodes[0].store[key];
            state.nodes.iter().any(|node| node.store[key] != first)
        })?;
        Some(format!(
            "nothing in flight but nodes disagree on k{key}: {}",
            state.summarize()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Update;

    #[test]
    fn passes_while_updates_are_in_flight() {
        let mut state = KvCluster::initial();
        state.nodes[0].store[1] = 2;
        state.nodes[1].inbox.push(Update {
            from: 0,
            key: 1,
            value: 2,
        });
        assert_eq!(Converged.check(&state), None);
    }

    #[test]
    fn fails_when_quiet_and_one_key_differs() {
        let mut state = KvCluster::initial();
        state.nodes[2].store[1] = 1;
        let reason = Converged.check(&state).expect("k1 differs");
        assert!(reason.contains("disagree on k1"));
    }

    #[test]
    fn passes_when_quiet_and_every_key_agrees() {
        let mut state = KvCluster::initial();
        for node in &mut state.nodes {
            node.store = [1, 2];
        }
        assert_eq!(Converged.check(&state), None);
    }
}
