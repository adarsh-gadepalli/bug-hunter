//! What can happen to the key-value cluster, and what each action does.
//!
//! A client write sets one key at one node and replicates `(key, value)` to the
//! other two nodes. Delivering an update merges its value into that key only.

use bughunter::SystemModel;

use crate::store::{KEYS, KvCluster, Update};

/// values a client may write. kept small so the state space stays finite.
pub const WRITE_VALUES: [u64; 2] = [1, 2];

/// max updates per inbox. a write is only offered while every other inbox has room.
pub const INBOX_CAP: usize = 2;

/// action enum - one step the explorer can take on a snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// a client sets `key` to `value` at `node`, which replicates it to the other nodes.
    Write { node: u8, key: u8, value: u64 },
    /// apply the oldest update in `nodes[to].inbox` to that node.
    Deliver { to: u8 },
}

/// merge enum - how a node combines a key's value with a delivered update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Merge {
    /// correct merge: `store[key] = max(store[key], value)`. delivery order cannot change the result.
    Max,
    /// planted bug: `store[key] = value`. the last update to arrive wins.
    Overwrite,
}

/// key-value model struct - the merge rule picks the fixed or the buggy variant.
#[derive(Clone, Copy, Debug)]
pub struct KvModel {
    pub merge: Merge,
}

/// this implements the SystemModel trait for the key-value model.
impl SystemModel for KvModel {
    type State = KvCluster;
    type Action = Action;

    /// initial state function - every key 0, every inbox empty.
    fn initial_state(&self) -> KvCluster {
        KvCluster::initial()
    }

    /// get actions function - every legal action from `state`.
    fn get_actions(&self, state: &KvCluster) -> Vec<Action> {
        let mut actions = Vec::new();
        for (index, node) in state.nodes.iter().enumerate() {
            let id = index as u8;
            if !node.inbox.is_empty() {
                actions.push(Action::Deliver { to: id });
            }
            if others_have_room(state, index) {
                for key in 0..KEYS as u8 {
                    for value in WRITE_VALUES {
                        actions.push(Action::Write {
                            node: id,
                            key,
                            value,
                        });
                    }
                }
            }
        }
        actions
    }

    /// transition function - returns the snapshot after `action`.
    /// delivering to an empty inbox returns an unchanged copy.
    fn transition(&self, state: &KvCluster, action: &Action) -> KvCluster {
        let mut next = state.clone();
        match *action {
            Action::Write { node, key, value } => {
                // the writer always takes the max, under both merge rules. only delivery is buggy.
                let writer = usize::from(node);
                let slot = &mut next.nodes[writer].store[usize::from(key)];
                *slot = (*slot).max(value);
                for (index, other) in next.nodes.iter_mut().enumerate() {
                    if index != writer {
                        other.inbox.push(Update {
                            from: node,
                            key,
                            value,
                        });
                    }
                }
            }
            Action::Deliver { to } => {
                let receiver = &mut next.nodes[usize::from(to)];
                if !receiver.inbox.is_empty() {
                    let update = receiver.inbox.remove(0);
                    let slot = &mut receiver.store[usize::from(update.key)];
                    *slot = match self.merge {
                        Merge::Max => (*slot).max(update.value),
                        Merge::Overwrite => update.value,
                    };
                }
            }
        }
        next
    }
}

// checks if every node other than `writer` has room in its inbox.
fn others_have_room(state: &KvCluster, writer: usize) -> bool {
    state
        .nodes
        .iter()
        .enumerate()
        .all(|(index, node)| index == writer || node.inbox.len() < INBOX_CAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUGGY: KvModel = KvModel {
        merge: Merge::Overwrite,
    };
    const FIXED: KvModel = KvModel { merge: Merge::Max };

    #[test]
    fn initial_state_offers_every_write() {
        // 3 nodes x 2 keys x 2 values, and nothing to deliver yet.
        let actions = FIXED.get_actions(&FIXED.initial_state());
        assert_eq!(actions.len(), 3 * KEYS * WRITE_VALUES.len());
    }

    #[test]
    fn write_sets_one_key_and_replicates_it() {
        let state = FIXED.transition(
            &FIXED.initial_state(),
            &Action::Write {
                node: 0,
                key: 1,
                value: 2,
            },
        );
        assert_eq!(state.nodes[0].store, [0, 2]);
        assert!(state.nodes[0].inbox.is_empty());
        for other in &state.nodes[1..] {
            assert_eq!(
                other.inbox,
                vec![Update {
                    from: 0,
                    key: 1,
                    value: 2
                }]
            );
        }
    }

    #[test]
    fn deliver_touches_only_the_updated_key() {
        let mut state = KvCluster::initial();
        state.nodes[1].store = [2, 0];
        state.nodes[1].inbox.push(Update {
            from: 0,
            key: 1,
            value: 1,
        });
        let next = FIXED.transition(&state, &Action::Deliver { to: 1 });
        assert_eq!(next.nodes[1].store, [2, 1]);
        assert!(next.nodes[1].inbox.is_empty());
    }

    #[test]
    fn overwrite_can_lower_a_key_but_max_cannot() {
        let mut state = KvCluster::initial();
        state.nodes[2].store = [2, 0];
        state.nodes[2].inbox.push(Update {
            from: 0,
            key: 0,
            value: 1,
        });
        let deliver = Action::Deliver { to: 2 };
        assert_eq!(FIXED.transition(&state, &deliver).nodes[2].store[0], 2);
        assert_eq!(BUGGY.transition(&state, &deliver).nodes[2].store[0], 1);
    }

    #[test]
    fn full_inbox_blocks_writes_from_other_nodes() {
        let mut state = KvCluster::initial();
        let update = Update {
            from: 0,
            key: 0,
            value: 1,
        };
        state.nodes[1].inbox = vec![update.clone(); INBOX_CAP];
        let actions = FIXED.get_actions(&state);
        // node 1 itself can still write, since its own inbox is not a target.
        assert!(actions.contains(&Action::Write {
            node: 1,
            key: 0,
            value: 1
        }));
        assert!(
            !actions
                .iter()
                .any(|action| matches!(action, Action::Write { node: 0 | 2, .. }))
        );
    }
}
