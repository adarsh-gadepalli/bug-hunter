//! Framework traits and a random-walk explorer. This crate never imports the dummy cluster.

mod model;
mod oracle;
mod rng;
mod state;
mod walk;

pub use model::SystemModel;
pub use oracle::{BugReport, Oracle};
pub use state::SystemState;
pub use walk::{Walk, random_walk};
