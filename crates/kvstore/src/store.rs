//! Snapshot of the key-value cluster: three nodes, each with a small store of
//! keys and a queue of replicated updates that have not been applied yet.

use bughunter::SystemState;

/// number of keys every node stores. keys are named k0, k1, ...
pub const KEYS: usize = 2;

/// node in the cluster.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// the value of each key, indexed by key number.
    pub store: [u64; KEYS],
    /// updates from other nodes that have not been applied to this node yet.
    pub inbox: Vec<Update>,
}

/// update struct - one replicated write: the sender, the key, and the new value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub from: u8,
    pub key: u8,
    pub value: u64,
}

/// key-value cluster state struct containing the nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KvCluster {
    pub nodes: [Node; 3],
}

impl KvCluster {
    /// initial state function - three nodes with every key at 0 and empty inboxes.
    pub fn initial() -> Self {
        Self {
            nodes: std::array::from_fn(|_| Node {
                store: [0; KEYS],
                inbox: Vec::new(),
            }),
        }
    }
}

/// this implements the SystemState trait for the key-value cluster.
impl SystemState for KvCluster {
    /// fingerprint function - hashes every key and every pending update.
    fn fingerprint(&self) -> u64 {
        let mut hash = Fnv::new();
        for node in &self.nodes {
            for value in node.store {
                hash.write_u64(value);
            }
            hash.write_u64(node.inbox.len() as u64);
            for update in &node.inbox {
                hash.write_u8(update.from);
                hash.write_u8(update.key);
                hash.write_u64(update.value);
            }
        }
        hash.finish()
    }

    /// summary function - e.g. `n0 k0=1 k1=0 [k1=2 from n2] | ...`.
    fn summarize(&self) -> String {
        let nodes: Vec<String> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let keys: Vec<String> = node
                    .store
                    .iter()
                    .enumerate()
                    .map(|(key, value)| format!("k{key}={value}"))
                    .collect();
                let inbox: Vec<String> = node
                    .inbox
                    .iter()
                    .map(|update| format!("k{}={} from n{}", update.key, update.value, update.from))
                    .collect();
                format!("n{index} {} [{}]", keys.join(" "), inbox.join(", "))
            })
            .collect();
        nodes.join(" | ")
    }
}

/// FNV-1a, the same hash as the dummy system uses.
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
        assert_eq!(
            KvCluster::initial().fingerprint(),
            KvCluster::initial().fingerprint()
        );
    }

    #[test]
    fn changing_one_key_changes_the_fingerprint() {
        let left = KvCluster::initial();
        let mut right = KvCluster::initial();
        right.nodes[1].store[1] = 1;
        assert_ne!(left.fingerprint(), right.fingerprint());
    }

    #[test]
    fn update_key_is_part_of_the_fingerprint() {
        let mut left = KvCluster::initial();
        let mut right = KvCluster::initial();
        left.nodes[0].inbox.push(Update {
            from: 1,
            key: 0,
            value: 2,
        });
        right.nodes[0].inbox.push(Update {
            from: 1,
            key: 1,
            value: 2,
        });
        assert_ne!(left.fingerprint(), right.fingerprint());
    }

    #[test]
    fn summary_shows_keys_and_inboxes() {
        let mut state = KvCluster::initial();
        state.nodes[1].inbox.push(Update {
            from: 0,
            key: 1,
            value: 2,
        });
        assert_eq!(
            state.summarize(),
            "n0 k0=0 k1=0 [] | n1 k0=0 k1=0 [k1=2 from n0] | n2 k0=0 k1=0 []"
        );
    }
}
