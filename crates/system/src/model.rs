//! What can happen to the dummy cluster, and what each action does.
//!
//! Each node keeps the largest value it has heard of. A client write raises
//! the writer's value and sends it to the other two nodes. Delivering a
//! message merges its payload into the receiver.

use bughunter::SystemModel;

use crate::system::{DummySystem, Message};

/// values a client may write. kept small so the state space stays finite.
pub const WRITE_VALUES: [u64; 2] = [1, 2];

/// max messages per inbox. a write is only offered while every other inbox has room.
/// without this cap, writes would outpace deliveries, inboxes would grow forever, and the
/// walk would almost never reach a state with nothing in flight.
pub const INBOX_CAP: usize = 2;

/// action enum - one step the explorer can take on a snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// a client writes `value` at `node`, which broadcasts it to the other nodes.
    Write { node: u8, value: u64 },
    /// hand the oldest message in `nodes[to].inbox` to that node.
    Deliver { to: u8 },
}

/// merge enum - how a node combines its value with a delivered payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Merge {
    /// correct merge: `local = max(local, payload)`. delivery order cannot change the result.
    Max,
    /// planted bug: `local = payload`. the last message to arrive wins, so two nodes
    /// that receive the same writes in a different order disagree.
    Overwrite,
}

/// dummy model struct - the merge rule picks the fixed or the buggy variant.
#[derive(Clone, Copy, Debug)]
pub struct DummyModel {
    pub merge: Merge,
}

/// this implements the SystemModel trait for the dummy model.
impl SystemModel for DummyModel {
    type State = DummySystem;
    type Action = Action;

    /// initial state function - three nodes with local data 0 and empty inboxes.
    fn initial_state(&self) -> DummySystem {
        DummySystem::initial()
    }

    /// get actions function - every legal action from `state`.
    fn get_actions(&self, state: &DummySystem) -> Vec<Action> {
        let mut actions = Vec::new();
        for (index, node) in state.nodes.iter().enumerate() {
            let id = index as u8;
            // a node can receive only if something is waiting in its inbox.
            if !node.inbox.is_empty() {
                actions.push(Action::Deliver { to: id });
            }
            // a node can write only if every other inbox has room for the broadcast.
            if others_have_room(state, index) {
                for value in WRITE_VALUES {
                    actions.push(Action::Write { node: id, value });
                }
            }
        }
        actions
    }

    /// transition function - returns the snapshot after `action`.
    /// delivering to an empty inbox returns an unchanged copy.
    fn transition(&self, state: &DummySystem, action: &Action) -> DummySystem {
        // copy the state so the input snapshot is left untouched.
        let mut next = state.clone();
        match *action {
            Action::Write { node, value } => {
                // the writer always takes the max, under both merge rules. only delivery is buggy.
                let writer = usize::from(node);
                next.nodes[writer].local = next.nodes[writer].local.max(value);
                // broadcast the write to every other node's inbox.
                for (index, other) in next.nodes.iter_mut().enumerate() {
                    if index != writer {
                        other.inbox.push(Message {
                            from: node,
                            payload: value,
                        });
                    }
                }
            }
            Action::Deliver { to } => {
                let receiver = &mut next.nodes[usize::from(to)];
                if !receiver.inbox.is_empty() {
                    // take the oldest message; inboxes are first in, first out.
                    let message = receiver.inbox.remove(0);
                    // merge the payload using the model's merge rule.
                    receiver.local = match self.merge {
                        Merge::Max => receiver.local.max(message.payload),
                        Merge::Overwrite => message.payload,
                    };
                }
            }
        }
        next
    }
}

// checks if other nodes have room in their inbox.
fn others_have_room(state: &DummySystem, writer: usize) -> bool {
    // iterate over the nodes and check if they have room in their inbox.
    state
        .nodes
        .iter()
        .enumerate()
        .all(|(index, node)| index == writer || node.inbox.len() < INBOX_CAP)
}