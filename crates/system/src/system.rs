//! Snapshot of the dummy cluster: three nodes, each with a term, a role,
//! and an inbox of messages that have not been delivered yet.

use bughunter::SystemState;

/// One process in the cluster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub term: u64,
    pub role: Role,
    /// Messages waiting to be delivered to this node.
    pub inbox: Vec<Envelope>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Follower = 0,
    Leader = 1,
}

/// A message sitting in some node's inbox. Nothing delivers it yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub from: u8,
    pub kind: MsgKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgKind {
    VoteRequest = 0,
    VoteGrant = 1,
}

/// Global state: every node's local fields plus in-flight messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummySystem {
    pub nodes: [Node; 3],
}

impl DummySystem {
    /// Three followers, term 0, empty inboxes.
    pub fn initial() -> Self {
        Self {
            nodes: std::array::from_fn(|_| Node {
                term: 0,
                role: Role::Follower,
                inbox: Vec::new(),
            }),
        }
    }
}

impl SystemState for DummySystem {
    fn fingerprint(&self) -> u64 {
        let mut hash = Fnv::new();
        for node in &self.nodes {
            hash.write_u64(node.term);
            hash.write_u8(node.role as u8);
            hash.write_u64(node.inbox.len() as u64);
            for message in &node.inbox {
                hash.write_u8(message.from);
                hash.write_u8(message.kind as u8);
            }
        }
        hash.finish()
    }
}

/// FNV-1a. Small enough to read, and the same bytes always produce the same `u64`.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_snapshots_share_a_fingerprint() {
        let left = DummySystem::initial();
        let right = DummySystem::initial();
        assert_eq!(left.fingerprint(), right.fingerprint());
    }

    #[test]
    fn changing_a_term_changes_the_fingerprint() {
        let left = DummySystem::initial();
        let mut right = DummySystem::initial();
        right.nodes[1].term = 1;
        assert_ne!(left.fingerprint(), right.fingerprint());
    }
}
