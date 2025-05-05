mod r#type;

pub mod batch;
pub mod communication;
pub mod flow;
pub mod node;

pub use anyhow::Result;
pub use r#type::{Action, DEFAULT_ACTION, ExecResult, NONE_ACTION, PrepResult};
