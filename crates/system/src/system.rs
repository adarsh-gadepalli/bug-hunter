//! Snapshot of a dummy cluster: three nodes, each with local data and a
//! queue of messages that have not been delivered yet.

use bughunter::SystemState;

/// One process in the cluster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// Opaque data this process currently stores.
    pub local: u64,
    /// Messages sent to this process that have not been delivered yet.
    pub inbox: Vec<Message>,
}

/// One in-flight message. `payload` is opaque: this snapshot does not
/// interpret it as a vote, a write, or any other operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub from: u8,
    pub payload: u64,
}

/// Global state: every node's local data plus in-flight messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummySystem {
    pub nodes: [Node; 3],
}

impl DummySystem {
    /// Three nodes with local data 0 and empty inboxes.
    pub fn initial() -> Self {
        Self {
            nodes: std::array::from_fn(|_| Node {
                local: 0,
                inbox: Vec::new(),
            }),
        }
    }
}

impl SystemState for DummySystem {
    fn fingerprint(&self) -> u64 {
        let mut hash = Fnv::new();
        for node in &self.nodes {
            hash.write_u64(node.local);
            hash.write_u64(node.inbox.len() as u64);
            for message in &node.inbox {
                hash.write_u8(message.from);
                hash.write_u64(message.payload);
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
    fn changing_local_data_changes_the_fingerprint() {
        let left = DummySystem::initial();
        let mut right = DummySystem::initial();
        right.nodes[1].local = 1;
        assert_ne!(left.fingerprint(), right.fingerprint());
    }
}
