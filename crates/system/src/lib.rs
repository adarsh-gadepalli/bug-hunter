//! Dummy 3-node cluster. This is the first system the framework explores.

mod model;
mod oracle;
mod system;

pub use model::{Action, DummyModel, INBOX_CAP, Merge, WRITE_VALUES};
pub use oracle::Converged;
pub use system::{DummySystem, Message, Node};
