mod r#type;

pub mod communication;
pub mod node;
// pub mod flow;

pub use anyhow::Result;
pub use r#type::{Action, DEFAULT_ACTION, ExecResult, NONE_ACTION, PrepResult};
