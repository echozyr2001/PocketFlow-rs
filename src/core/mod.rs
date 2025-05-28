mod r#type;

pub mod batch;
pub mod communication;
pub mod flow;
pub mod node;

pub use anyhow::Result;
pub use r#type::{Action, DEFAULT_ACTION, ExecResult, NONE_ACTION, PostResult, PrepResult};
pub use communication::BaseSharedStore as Store;

// NodeId 类型定义 - 用于标识流程中的节点
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
