//! Dummy 3-node cluster. This is the first system the framework will explore.

mod system;

pub use system::{DummySystem, Envelope, MsgKind, Node, Role};
