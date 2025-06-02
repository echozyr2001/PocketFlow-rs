//! # Node System - The Building Blocks of PocketFlow
//!
//! This module provides the node system, which represents the fundamental computation
//! units in PocketFlow workflows. Nodes are designed around a three-phase execution
//! model that separates concerns and enables robust, retry-safe operations.
//!
//! ## Three-Phase Execution Model
//!
//! Every node follows a consistent execution pattern:
//!
//! ### 1. Prep Phase (`prep`)
//! - **Purpose**: Read and validate input data from the shared store
//! - **Safety**: Read-only access to shared store, safe to retry
//! - **Responsibility**: Data validation, input preparation, parameter extraction
//! - **Output**: Prepared data for the execution phase
//!
//! ### 2. Exec Phase (`exec`)
//! - **Purpose**: Perform the main computation (LLM calls, API requests, processing)
//! - **Safety**: Idempotent by design, no shared store access
//! - **Responsibility**: Core business logic, external service calls, computations
//! - **Output**: Computation results for post-processing
//!
//! ### 3. Post Phase (`post`)
//! - **Purpose**: Write results back to shared store and determine next action
//! - **Safety**: Controlled shared store mutations, error handling
//! - **Responsibility**: Result storage, state updates, flow control decisions
//! - **Output**: Action indicating next flow step
//!
//! ## Core Components
//!
//! ### NodeBackend Trait
//! The fundamental abstraction for implementing custom nodes:
//!
//! ```rust
//! use pocketflow_rs::prelude::*;
//! use async_trait::async_trait;
//!
//! struct CustomNode;
//!
//! #[async_trait]
//! impl<S: StorageBackend + Send + Sync> NodeBackend<S> for CustomNode {
//!     type PrepResult = String;
//!     type ExecResult = String;
//!     type Error = Box<dyn std::error::Error + Send + Sync>;
//!     
//!     async fn prep(&mut self, store: &SharedStore<S>, _: &ExecutionContext)
//!         -> Result<Self::PrepResult, Self::Error> {
//!         // Read input from shared store
//!         let input = store.get("input")?.unwrap_or_default();
//!         Ok(input.to_string())
//!     }
//!     
//!     async fn exec(&mut self, prep_result: Self::PrepResult, _: &ExecutionContext)
//!         -> Result<Self::ExecResult, Self::Error> {
//!         // Perform computation (idempotent)
//!         Ok(format!("processed: {}", prep_result))
//!     }
//!     
//!     async fn post(&mut self, store: &mut SharedStore<S>, _prep: Self::PrepResult,
//!                   exec_result: Self::ExecResult, _: &ExecutionContext)
//!         -> Result<Action, Self::Error> {
//!         // Write results and determine next action
//!         store.set("output".to_string(), serde_json::json!(exec_result))?;
//!         Ok(Action::simple("continue"))
//!     }
//! }
//! ```
//!
//! ### Node Wrapper
//! A concrete implementation that wraps any `NodeBackend` and provides:
//! - Automatic retry logic with configurable delays
//! - Error handling and fallback mechanisms
//! - Execution context management
//! - Lifecycle coordination
//!
//! ### ExecutionContext
//! Provides execution metadata and controls:
//! - **Retry Management**: Current attempt, max retries, delays
//! - **Unique Identification**: Execution IDs for tracking and correlation
//! - **Metadata Storage**: Additional context data for complex flows
//! - **Flow Coordination**: Cross-node communication and state
//!
//! ## Built-in Node Types
//!
//! The library provides several ready-to-use node implementations:
//!
//! ### Basic Nodes (feature: `builtin-nodes`)
//! - **LogNode**: Simple logging with configurable output
//! - **SetValueNode**: Write values to shared store
//! - **GetValueNode**: Read and validate shared store values  
//! - **DelayNode**: Configurable execution delays
//! - **ConditionalNode**: Branching logic based on store state
//!
//! ### LLM Nodes (feature: `builtin-llm`)
//! - **ApiRequestNode**: Configurable HTTP API calls with streaming support
//! - **MockLlmNode**: Testing and development placeholder
//!
//! ## Advanced Features
//!
//! ### FunctionNode
//! For rapid prototyping, create nodes from closures:
//!
//! ```rust
//! # use pocketflow_rs::prelude::*;
//! # use std::time::Duration;
//! let quick_node = FunctionNode::new(
//!     "processor".to_string(),
//!     |store, _ctx| store.get("input").unwrap().unwrap_or_default(),
//!     |input, _ctx| Ok(format!("Quick processing: {}", input)),
//!     |store, _prep, result, _ctx| {
//!         store.set("quick_output".to_string(), serde_json::json!(result))?;
//!         Ok(Action::simple("done"))
//!     }
//! ).with_retries(3).with_retry_delay(Duration::from_millis(100));
//! ```
//!
//! ### Error Handling
//! Comprehensive error system supporting:
//! - **Automatic Retries**: Configurable retry counts and delays
//! - **Graceful Fallbacks**: Custom error recovery strategies
//! - **Error Classification**: Different error types for different handling
//! - **Context Preservation**: Error context carried through execution
//!
//! ## Design Principles
//!
//! 1. **Separation of Concerns**: Each phase has distinct responsibilities
//! 2. **Retry Safety**: Phases designed for safe retry on failure
//! 3. **Composability**: Nodes work together seamlessly in complex flows
//! 4. **Observability**: Rich context and state tracking throughout execution
//! 5. **Performance**: Minimal allocations, efficient async execution
//! 6. **Extensibility**: Easy to implement custom node types

use crate::{Action, PocketFlowError, PocketFlowResult, SharedStore, StorageBackend};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

// Type aliases to reduce complexity warnings
type PrepFn<S, P> = Box<dyn Fn(&SharedStore<S>, &ExecutionContext) -> P + Send + Sync>;
type ExecFn<P, E> = Box<
    dyn Fn(P, &ExecutionContext) -> Result<E, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;
type PostFn<S, P, E> = Box<
    dyn Fn(
            &mut SharedStore<S>,
            P,
            E,
            &ExecutionContext,
        ) -> Result<Action, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

/// Simple error type for Node operations
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Preparation error: {0}")]
    PrepError(String),
}

impl From<String> for NodeError {
    fn from(s: String) -> Self {
        NodeError::ExecutionError(s)
    }
}

impl From<&str> for NodeError {
    fn from(s: &str) -> Self {
        NodeError::ExecutionError(s.to_string())
    }
}

/// Represents the execution context for a node, containing the current retry count
/// and other execution metadata.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Current retry attempt (0-based)
    pub current_retry: usize,
    /// Maximum number of retries allowed
    pub max_retries: usize,
    /// Wait duration between retries
    pub retry_delay: Duration,
    /// Unique execution ID for tracking
    pub execution_id: String,
    /// Additional metadata for the execution
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(max_retries: usize, retry_delay: Duration) -> Self {
        Self {
            current_retry: 0,
            max_retries,
            retry_delay,
            execution_id: uuid::Uuid::new_v4().to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Check if more retries are available
    pub fn can_retry(&self) -> bool {
        self.current_retry < self.max_retries
    }

    /// Increment retry count
    pub fn next_retry(&mut self) {
        self.current_retry += 1;
    }

    /// Get the execution ID
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Remove metadata value
    pub fn remove_metadata(&mut self, key: &str) -> Option<serde_json::Value> {
        self.metadata.remove(key)
    }

    /// Get all metadata
    pub fn metadata(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

/// Core trait for implementing custom node backends.
///
/// A Node represents the smallest building block in PocketFlow workflows.
/// Each node has three execution phases:
/// 1. `prep` - Read and preprocess data from shared store
/// 2. `exec` - Execute compute logic (LLM calls, APIs, etc.)
/// 3. `post` - Postprocess and write results back to shared store
#[async_trait]
pub trait NodeBackend<S: StorageBackend>: Send + Sync {
    /// The type returned by the prep phase
    type PrepResult: Send + Sync + Clone + 'static;
    /// The type returned by the exec phase  
    type ExecResult: Send + Sync + 'static;
    /// Error type for this node
    type Error: std::error::Error + Send + Sync + 'static;

    /// Preparation phase: read and preprocess data from shared store
    ///
    /// This phase should:
    /// - Read necessary data from the shared store
    /// - Validate inputs
    /// - Prepare data for the execution phase
    ///
    /// Returns data that will be passed to `exec()`
    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error>;

    /// Execution phase: perform the main computation
    ///
    /// This phase should:
    /// - Perform the core logic (LLM calls, API requests, etc.)
    /// - Be idempotent (safe to retry)
    /// - NOT access the shared store directly
    ///
    /// Returns data that will be passed to `post()`
    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error>;

    /// Post-processing phase: write results back to shared store
    ///
    /// This phase should:
    /// - Write results to the shared store
    /// - Update state
    /// - Determine the next action
    ///
    /// Returns the action to take next
    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error>;

    /// Fallback handler for when exec() fails after all retries
    ///
    /// Override this to provide graceful error handling instead of propagating errors.
    /// By default, this re-raises the error.
    async fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Err(error)
    }

    /// Get the node's name/identifier for logging and debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Get maximum number of retries for this node
    fn max_retries(&self) -> usize {
        1 // Default: no retries
    }

    /// Get retry delay for this node
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(0) // Default: no delay
    }
}

/// A concrete Node implementation that wraps a NodeBackend
pub struct Node<B, S>
where
    B: NodeBackend<S>,
    S: StorageBackend,
{
    backend: B,
    _phantom: std::marker::PhantomData<S>,
}

impl<B, S> Node<B, S>
where
    B: NodeBackend<S>,
    S: StorageBackend,
{
    /// Create a new node with the given backend
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Run the complete node execution cycle: prep -> exec -> post
    pub async fn run(&mut self, store: &mut SharedStore<S>) -> PocketFlowResult<Action> {
        let context = ExecutionContext::new(self.backend.max_retries(), self.backend.retry_delay());

        // Prep phase
        let prep_result = self
            .backend
            .prep(store, &context)
            .await
            .map_err(|e| PocketFlowError::ExecutionError(format!("Prep failed: {}", e)))?;

        // Exec phase with retries
        let exec_result = self
            .exec_with_retries(prep_result.clone(), context.clone())
            .await
            .map_err(|e| PocketFlowError::ExecutionError(format!("Exec failed: {}", e)))?;

        // Post phase
        let action = self
            .backend
            .post(store, prep_result, exec_result, &context)
            .await
            .map_err(|e| PocketFlowError::ExecutionError(format!("Post failed: {}", e)))?;

        Ok(action)
    }

    /// Execute the exec phase with retry logic
    async fn exec_with_retries(
        &mut self,
        prep_result: B::PrepResult,
        mut context: ExecutionContext,
    ) -> Result<B::ExecResult, B::Error> {
        loop {
            match self.backend.exec(prep_result.clone(), &context).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if context.can_retry() {
                        // Wait before retry
                        if context.retry_delay > Duration::ZERO {
                            sleep(context.retry_delay).await;
                        }
                        context.next_retry();
                        continue;
                    } else {
                        // All retries exhausted, try fallback
                        match self
                            .backend
                            .exec_fallback(prep_result, error, &context)
                            .await
                        {
                            Ok(result) => return Ok(result),
                            Err(fallback_error) => {
                                return Err(fallback_error);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get the underlying backend
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Get mutable reference to the underlying backend
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

// Convenience type aliases for common node types
pub type InMemoryNode<B> = Node<B, crate::storage::InMemoryStorage>;

/// Builder for creating nodes with custom configuration
pub struct NodeBuilder<B> {
    backend: B,
}

impl<B> NodeBuilder<B> {
    /// Create a new node builder with the given backend
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Build the final node
    pub fn build<S: StorageBackend>(self) -> Node<B, S>
    where
        B: NodeBackend<S>,
    {
        Node::new(self.backend)
    }
}

/// A simple function-based node implementation for quick prototyping
pub struct FunctionNode<S, P, E>
where
    S: StorageBackend,
    P: Send + Sync + Clone + 'static,
    E: Send + Sync + 'static,
{
    name: String,
    prep_fn: PrepFn<S, P>,
    exec_fn: ExecFn<P, E>,
    post_fn: PostFn<S, P, E>,
    max_retries: usize,
    retry_delay: Duration,
}

impl<S, P, E> FunctionNode<S, P, E>
where
    S: StorageBackend,
    P: Send + Sync + Clone + 'static,
    E: Send + Sync + 'static,
{
    /// Create a new function-based node
    pub fn new<PrepFn, ExecFn, PostFn>(
        name: String,
        prep_fn: PrepFn,
        exec_fn: ExecFn,
        post_fn: PostFn,
    ) -> Self
    where
        PrepFn: Fn(&SharedStore<S>, &ExecutionContext) -> P + Send + Sync + 'static,
        ExecFn: Fn(P, &ExecutionContext) -> Result<E, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
        PostFn: Fn(
                &mut SharedStore<S>,
                P,
                E,
                &ExecutionContext,
            ) -> Result<Action, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            name,
            prep_fn: Box::new(prep_fn),
            exec_fn: Box::new(exec_fn),
            post_fn: Box::new(post_fn),
            max_retries: 1,
            retry_delay: Duration::from_secs(0),
        }
    }

    /// Set maximum number of retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }
}

#[async_trait]
impl<S, P, E> NodeBackend<S> for FunctionNode<S, P, E>
where
    S: StorageBackend + Send + Sync,
    P: Send + Sync + Clone + 'static,
    E: Send + Sync + 'static,
{
    type PrepResult = P;
    type ExecResult = E;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok((self.prep_fn)(store, context))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        (self.exec_fn)(prep_result, context).map_err(|e| NodeError::ExecutionError(e.to_string()))
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        (self.post_fn)(store, prep_result, exec_result, context)
            .map_err(|e| NodeError::ExecutionError(e.to_string()))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }

    fn retry_delay(&self) -> Duration {
        self.retry_delay
    }
}

pub mod builtin;

#[cfg(test)]
mod tests;
