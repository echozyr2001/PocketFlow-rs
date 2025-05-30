//! # PocketFlow-RS
//! 
//! A minimalist LLM framework in Rust, inspired by the TypeScript version of PocketFlow.
//! 
//! PocketFlow models LLM workflows as a **Graph + Shared Store**:
//! - **Node**: Handles simple (LLM) tasks
//! - **Flow**: Connects nodes through **Actions** (labeled edges)  
//! - **Shared Store**: Enables communication between nodes within flows
//! 
//! ## Core Principles
//! 
//! - **Lightweight**: Minimal core abstraction with zero vendor lock-in
//! - **Expressive**: Support for Agents, Workflows, RAG, and more
//! - **Extensible**: Simple core that's easy to build upon
//! 
//! ## Example
//! 
//! ```rust
//! use pocketflow_rs::{SharedStore, Action};
//! use serde_json::json;
//! 
//! let mut store = SharedStore::new();
//! store.set("input".to_string(), json!("Hello, PocketFlow!"));
//! 
//! let action: Action = "continue".into();
//! ```

pub mod action;
pub mod shared_store;
pub mod storage;
pub mod node;
pub mod flow;

// Re-export core types for easier access
pub use action::{Action, ActionCondition, ActionBuilder, ComparisonOperator};
pub use shared_store::{SharedStore, InMemorySharedStore};
pub use storage::{StorageBackend, InMemoryStorage, FileStorage};
pub use node::{NodeBackend, ExecutionContext, Node, InMemoryNode, NodeBuilder, FunctionNode};
pub use node::{LogNode, SetValueNode, GetValueNode, ConditionalNode, DelayNode, MockLlmNode, ApiRequestNode, ApiConfig};
pub use flow::{Flow, BasicFlow, FlowBuilder, FlowConfig, FlowError, FlowExecutionResult, Route, RouteCondition, FlowNode};

// Re-export commonly used external types
pub use serde_json::Value as JsonValue;

/// Convenient re-exports for common types and traits
pub mod prelude {
    pub use crate::{
        Action, ActionBuilder, ActionCondition, ComparisonOperator,
        SharedStore, StorageBackend, PocketFlowResult, PocketFlowError,
    };
    pub use crate::storage::{InMemoryStorage, FileStorage};
    pub use crate::node::{
        Node, NodeBackend, ExecutionContext, NodeError, NodeBuilder, 
        FunctionNode, InMemoryNode
    };
    pub use crate::node::builtin::*;
    pub use crate::flow::{
        Flow, BasicFlow, FlowBuilder, FlowConfig, FlowError, 
        FlowExecutionResult, Route, RouteCondition
    };
}

/// Result type alias for PocketFlow operations
pub type PocketFlowResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Common error type for PocketFlow operations
#[derive(Debug)]
pub enum PocketFlowError {
    /// Error during serialization/deserialization
    SerializationError(serde_json::Error),
    /// Error when a required key is not found in SharedStore
    KeyNotFound(String),
    /// Error during node execution
    ExecutionError(String),
    /// Error during flow orchestration
    FlowError(String),
}

impl std::fmt::Display for PocketFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PocketFlowError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            PocketFlowError::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            PocketFlowError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            PocketFlowError::FlowError(msg) => write!(f, "Flow error: {}", msg),
        }
    }
}

impl std::error::Error for PocketFlowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PocketFlowError::SerializationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for PocketFlowError {
    fn from(error: serde_json::Error) -> Self {
        PocketFlowError::SerializationError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_integration() {
        let mut store = InMemorySharedStore::new();
        let action: Action = "test_action".into(); // Use .into() for conversion
        
        // Test setting and getting values
        store.set("test_key".to_string(), json!("test_value")).unwrap();
        assert_eq!(store.get("test_key").unwrap(), Some(json!("test_value")));
        
        // Test action usage
        assert_eq!(action.name(), "test_action"); // Compare names instead of direct equality
    }
    
    #[test]
    fn test_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());
        
        let pf_error: PocketFlowError = json_error.unwrap_err().into();
        assert!(matches!(pf_error, PocketFlowError::SerializationError(_)));
    }
}