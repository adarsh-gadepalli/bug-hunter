//! A replicated key-value store: three nodes, each holding a few keys, that
//! replicate every client write to each other.

mod model;
mod oracle;
mod store;

pub use model::{Action, INBOX_CAP, KvModel, Merge, WRITE_VALUES};
pub use oracle::Converged;
pub use store::{KEYS, KvCluster, Node, Update};
