//! # PocketFlow-RS
//!
//! A minimalist LLM framework in Rust, inspired by the TypeScript version of PocketFlow.
//!
//! PocketFlow models LLM workflows as a **Graph + Shared Store**:
//! - **Node**: Handles simple (LLM) tasks
//! - **Flow**: Connects nodes through **Actions** (labeled edges)  
//! - **Shared Store**: Enables communication between nodes within flows
//!
//! ## 🏗️ Feature Architecture
//!
//! PocketFlow-RS is organized into several feature modules:
//!
//! ### Core Features
//! - `core`: Essential types (Action, SharedStore, Node, Flow)
//! - `async`: Async support (AsyncSharedStore)
//!
//! ### Built-in Components  
//! - `builtin-nodes`: Basic nodes (LogNode, SetValueNode, etc.)
//! - `builtin-llm`: LLM-related nodes (MockLlmNode, ApiRequestNode)
//! - `builtin-flows`: Advanced flow components (FlowNode)
//! - `builtin`: All built-in components
//!
//! ### Storage Backends
//! - `storage-memory`: In-memory storage (included in core)
//! - `storage-file`: File-based storage
//! - `storage-redis`: Redis backend
//! - `storage-database`: SQL databases via SeaORM
//! - `storage-sqlite`: SQLite support
//! - `storage-postgres`: PostgreSQL support  
//! - `storage-mysql`: MySQL support
//! - `storage-all`: All storage backends
//!
//! ### Convenience Features
//! - `default`: Core + async + builtin-nodes + storage-memory
//! - `full`: Complete feature set
//! - `dev`: Development configuration
//!
//! ## 🚀 Quick Start
//!
//! ### Minimal Core
//! ```toml
//! pocketflow-rs = { version = "0.1", default-features = false, features = ["core"] }
//! ```
//!
//! ### Default Configuration
//! ```toml
//! pocketflow-rs = "0.1"  # Includes core, async, builtin-nodes, storage-memory
//! ```
//!
//! ### Full Features
//! ```toml
//! pocketflow-rs = { version = "0.1", features = ["full"] }
//! ```
//!
//! ## Example
//!
//! ```rust
//! use pocketflow_rs::prelude::*;
//! use serde_json::json;
//!
//! #[cfg(feature = "core")]
//! {
//!     let mut store = SharedStore::new();
//!     store.set("input".to_string(), json!("Hello, PocketFlow!"));
//!     
//!     let action: Action = "continue".into();
//! }
//! ```

// ============================================================================
// CORE MODULES (always available)
// ============================================================================

pub mod action;
pub mod flow;
pub mod node;
pub mod shared_store;
pub mod storage;

// ============================================================================
// CORE RE-EXPORTS
// ============================================================================

// Action system - always available
pub use action::{Action, ActionBuilder, ActionCondition, ComparisonOperator};

// SharedStore - always available
pub use shared_store::{AsyncSharedStore, InMemorySharedStore, SharedStore};

// Storage traits - always available
pub use storage::StorageBackend;

// Node system - always available
pub use node::{ExecutionContext, FunctionNode, InMemoryNode, Node, NodeBackend, NodeBuilder};

// Flow system - always available
pub use flow::{
    BasicFlow, Flow, FlowBuilder, FlowConfig, FlowError, FlowExecutionResult, Route, RouteCondition,
};

// ============================================================================
// STORAGE BACKEND RE-EXPORTS (feature-gated)
// ============================================================================

/// Memory storage (included with core)
#[cfg(feature = "storage-memory")]
pub use storage::{InMemoryStorage, InMemoryStorageError};

/// File storage
#[cfg(feature = "storage-file")]
pub use storage::FileStorage;

/// Redis storage
#[cfg(feature = "storage-redis")]
pub use storage::RedisStorage;

/// Database storage  
#[cfg(feature = "storage-database")]
pub use storage::DatabaseStorage;

// ============================================================================
// BUILTIN COMPONENTS RE-EXPORTS (feature-gated)
// ============================================================================

/// Basic builtin nodes
#[cfg(feature = "builtin-nodes")]
pub use node::builtin::{ConditionalNode, DelayNode, GetValueNode, LogNode, SetValueNode};

/// LLM-related nodes
#[cfg(feature = "builtin-llm")]
pub use node::builtin::{ApiConfig, ApiRequestNode, MockLlmNode};

/// Flow components
#[cfg(feature = "builtin-flows")]
pub use flow::FlowNode;

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

/// Commonly used external types
pub use serde_json::Value as JsonValue;

/// Convenient re-exports for common types and traits
pub mod prelude {
    // Core types - always available
    pub use crate::{
        Action, ActionBuilder, ActionCondition, ComparisonOperator, ExecutionContext, Flow,
        FlowBuilder, FlowError, FunctionNode, Node, NodeBackend, NodeBuilder, PocketFlowError,
        PocketFlowResult, RouteCondition, SharedStore, StorageBackend,
    };

    // Storage backends - feature-gated
    #[cfg(feature = "storage-memory")]
    pub use crate::storage::{InMemoryStorage, InMemoryStorageError};

    #[cfg(feature = "storage-file")]
    pub use crate::storage::FileStorage;

    #[cfg(feature = "storage-redis")]
    pub use crate::storage::RedisStorage;

    #[cfg(feature = "storage-database")]
    pub use crate::storage::DatabaseStorage;

    // Async support - always available
    pub use crate::shared_store::AsyncSharedStore;

    // Builtin nodes - feature-gated
    #[cfg(feature = "builtin-nodes")]
    pub use crate::node::builtin::{
        ConditionalNode, DelayNode, GetValueNode, LogNode, SetValueNode,
    };

    // LLM nodes - feature-gated
    #[cfg(feature = "builtin-llm")]
    pub use crate::node::builtin::{ApiConfig, ApiRequestNode, MockLlmNode};

    // Flow components - feature-gated
    #[cfg(feature = "builtin-flows")]
    pub use crate::flow::FlowNode;

    // Commonly used external types
    pub use serde_json::Value as JsonValue;
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

/// Result type alias for PocketFlow operations
pub type PocketFlowResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Common error type for PocketFlow operations
#[derive(Debug, thiserror::Error)]
pub enum PocketFlowError {
    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Error when a required key is not found in SharedStore
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Error during node execution
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Error during flow orchestration
    #[error("Flow error: {0}")]
    FlowError(String),

    /// Feature not enabled
    #[error("Feature not enabled: {0}. Please enable the required feature flag.")]
    FeatureNotEnabled(String),
}

impl PocketFlowError {
    /// Create a new feature not enabled error
    pub fn feature_not_enabled(feature: &str) -> Self {
        Self::FeatureNotEnabled(feature.to_string())
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_integration() {
        let mut store = InMemorySharedStore::new();
        let action: Action = "test_action".into();

        // Test setting and getting values
        store
            .set("test_key".to_string(), json!("test_value"))
            .unwrap();
        assert_eq!(store.get("test_key").unwrap(), Some(json!("test_value")));

        // Test action usage
        assert_eq!(action.name(), "test_action");
    }

    #[test]
    fn test_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());

        let pf_error: PocketFlowError = json_error.unwrap_err().into();
        assert!(matches!(pf_error, PocketFlowError::SerializationError(_)));
    }

    #[test]
    fn test_feature_not_enabled_error() {
        let error = PocketFlowError::feature_not_enabled("storage-redis");
        assert_eq!(
            error.to_string(),
            "Feature not enabled: storage-redis. Please enable the required feature flag."
        );
    }
}
