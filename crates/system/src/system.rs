//! Snapshot of a dummy cluster: three nodes, each with local data and a
//! queue of messages that have not been delivered yet.

use bughunter::SystemState;

/// node in the cluster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// blackbox data variable
    pub local: u64,
    /// a list of messages that have not been delivered to this node yet.
    pub inbox: Vec<Message>,
}

/// message struct containing the sender and the payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub from: u8,
    pub payload: u64,
}

/// dummy system state struct containing the nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummySystem {
    pub nodes: [Node; 3],
}

impl DummySystem {
    /// initial state function - three nodes with local data 0 and empty inboxes.
    pub fn initial() -> Self {
        Self {
            nodes: std::array::from_fn(|_| Node {
                local: 0,
                inbox: Vec::new(),
            }),
        }
    }
}

/// this implements the SystemState trait for the dummy system.
impl SystemState for DummySystem {

    // fingerprint function - hashes the state of the system
    fn fingerprint(&self) -> u64 {
        // fnv hashing 
        let mut hash = Fnv::new();

        // iterate over the nodes and hash the local data and the inbox length and the messages
        for node in &self.nodes {
            hash.write_u64(node.local);
            hash.write_u64(node.inbox.len() as u64);
            for message in &node.inbox {
                hash.write_u8(message.from);
                hash.write_u64(message.payload);
            }
        }

        // return the finished hash.
        hash.finish()
    }

    /// summary function - returns a string representation of the state of the system.
    fn summarize(&self) -> String {
        let nodes: Vec<String> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let inbox: Vec<String> = node
                    .inbox
                    .iter()
                    .map(|message| format!("{} from n{}", message.payload, message.from))
                    .collect();
                format!("n{index}={} [{}]", node.local, inbox.join(", "))
            })
            .collect();
        nodes.join(" | ")
    }
}

/// FNV-1a. Small enough to read, and the same bytes always produce the same `u64`.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        // initial hash value, defined by the fnv hash function.
        Self(0xcbf29ce484222325)
    }

    /// fnv hashing logic 
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
        // returns the final hash value.
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

    #[test]
    fn summary_shows_values_and_inboxes() {
        let mut state = DummySystem::initial();
        state.nodes[1].inbox.push(Message {
            from: 0,
            payload: 2,
        });
        assert_eq!(state.summarize(), "n0=0 [] | n1=0 [2 from n0] | n2=0 []");
    }
}
