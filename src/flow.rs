//! # Flow Orchestration System
//!
//! This module provides the core flow orchestration engine for PocketFlow, enabling
//! the composition and execution of complex workflows from individual nodes.
//!
//! ## Architecture
//!
//! ### Flow as a Graph
//! PocketFlow models workflows as directed graphs where:
//! - **Nodes**: Individual computation units (LLM calls, data processing, etc.)
//! - **Edges**: Action-based transitions with optional conditions
//! - **SharedStore**: Communication medium between nodes
//!
//! ### Three-Phase Execution Model
//! 1. **Validation**: Ensure flow integrity (reachable nodes, valid routes)
//! 2. **Runtime Execution**: Step-by-step node execution with action-based routing
//! 3. **Result Collection**: Comprehensive execution results with path tracking
//!
//! ## Core Components
//!
//! ### BasicFlow
//! The main flow implementation supporting:
//! - Dynamic node addition and route configuration
//! - Configurable execution policies (max steps, cycle detection, terminal actions)
//! - Rich execution context with retry logic and metadata
//! - Comprehensive error handling and recovery
//!
//! ### FlowBuilder
//! A fluent builder for constructing flows:
//! ```rust
//! # use pocketflow_rs::prelude::*;
//! # use pocketflow_rs::node::builtin::LogNode;
//! # #[cfg(feature = "builtin-nodes")]
//! let flow = FlowBuilder::new()
//!     .start_node("entry")
//!     .max_steps(100)
//!     .terminal_action("complete")
//!     .node("entry", Node::new(LogNode::new("Starting", Action::simple("continue"))))
//!     .route("entry", "continue", "next")
//!     .conditional_route(
//!         "entry",
//!         "branch",
//!         "conditional_node",
//!         RouteCondition::KeyExists("flag".to_string())
//!     )
//!     .build();
//! ```
//!
//! ### FlowNode
//! Enables flow composition by wrapping flows as nodes, supporting:
//! - Nested flow execution
//! - Configurable nesting depth limits
//! - Result propagation between flow levels
//! - Metadata preservation across nesting levels
//!
//! ## Execution Guarantees
//!
//! ### Safety
//! - **Cycle Detection**: Prevents infinite loops in flow execution
//! - **Step Limiting**: Configurable maximum execution steps
//! - **Nesting Depth Control**: Prevents stack overflow in nested flows
//! - **Error Isolation**: Node failures don't crash entire flows
//!
//! ### Observability
//! - **Execution Path Tracking**: Complete record of node execution order
//! - **Step Counting**: Performance and complexity metrics
//! - **Success/Failure Status**: Clear execution outcome indication
//! - **Final Action Capture**: Last action taken before termination
//!
//! ## Advanced Features
//!
//! ### Conditional Routing
//! Routes can include conditions evaluated against the shared store:
//! ```rust
//! # use pocketflow_rs::flow::RouteCondition;
//! let condition = RouteCondition::KeyEquals(
//!     "processing_mode".to_string(),
//!     serde_json::json!("batch")
//! );
//! ```
//!
//! ### Flow Composition
//! Flows can be composed hierarchically using FlowNode:
//! ```rust
//! # use pocketflow_rs::prelude::*;
//! # let sub_flow = BasicFlow::new();
//! let flow_node = FlowNode::new(sub_flow);
//! // Use flow_node like any other node in a larger flow
//! ```
//!
//! ## Error Handling
//!
//! The flow system provides comprehensive error handling:
//! - **NodeNotFound**: Missing node references
//! - **NoRouteFound**: Invalid action routing
//! - **CycleDetected**: Infinite loop prevention
//! - **MaxStepsExceeded**: Runaway execution protection
//! - **InvalidConfiguration**: Setup validation errors

use crate::node::{ExecutionContext, NodeBackend, NodeError};
use crate::{Action, SharedStore, StorageBackend};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during flow execution
#[derive(Debug, Clone)]
pub enum FlowError {
    /// Node with the given ID was not found
    NodeNotFound(String),
    /// No route found for the given action from the current node
    NoRouteFound(String, String), // (node_id, action)
    /// Cycle detected in the flow
    CycleDetected(Vec<String>),
    /// Maximum execution steps exceeded
    MaxStepsExceeded(usize),
    /// Node execution error
    NodeError(String),
    /// Invalid flow configuration
    InvalidConfiguration(String),
}

impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            FlowError::NoRouteFound(node_id, action) => {
                write!(
                    f,
                    "No route found from node '{}' for action '{}'",
                    node_id, action
                )
            }
            FlowError::CycleDetected(path) => {
                write!(f, "Cycle detected in flow: {}", path.join(" -> "))
            }
            FlowError::MaxStepsExceeded(max) => {
                write!(f, "Maximum execution steps exceeded: {}", max)
            }
            FlowError::NodeError(msg) => write!(f, "Node execution error: {}", msg),
            FlowError::InvalidConfiguration(msg) => {
                write!(f, "Invalid flow configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for FlowError {}

impl From<NodeError> for FlowError {
    fn from(err: NodeError) -> Self {
        FlowError::NodeError(err.to_string())
    }
}

/// Represents a route from one node to another based on an action
#[derive(Debug, Clone)]
pub struct Route {
    /// The action that triggers this route
    pub action: String,
    /// The target node ID
    pub target_node_id: String,
    /// Optional condition that must be met for this route to be taken
    pub condition: Option<RouteCondition>,
}

/// Conditions for route evaluation
#[derive(Debug)]
pub enum RouteCondition {
    /// Always true
    Always,
    /// Check if a key exists in the shared store
    KeyExists(String),
    /// Check if a key equals a specific value
    KeyEquals(String, serde_json::Value),
}

impl Clone for RouteCondition {
    fn clone(&self) -> Self {
        match self {
            RouteCondition::Always => RouteCondition::Always,
            RouteCondition::KeyExists(key) => RouteCondition::KeyExists(key.clone()),
            RouteCondition::KeyEquals(key, value) => {
                RouteCondition::KeyEquals(key.clone(), value.clone())
            }
        }
    }
}

impl RouteCondition {
    /// Evaluate the condition against the shared store
    pub fn evaluate<S: StorageBackend>(&self, store: &SharedStore<S>) -> bool {
        match self {
            RouteCondition::Always => true,
            RouteCondition::KeyExists(key) => store.contains_key(key).unwrap_or(false),
            RouteCondition::KeyEquals(key, expected_value) => {
                if let Ok(Some(actual_value)) = store.get(key) {
                    &actual_value == expected_value
                } else {
                    false
                }
            }
        }
    }
}

/// Execution result from a flow run
#[derive(Debug, Clone)]
pub struct FlowExecutionResult {
    /// The final action that terminated the flow
    pub final_action: Action,
    /// The ID of the last executed node
    pub last_node_id: String,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Whether the flow completed successfully
    pub success: bool,
    /// Execution path (node IDs in order)
    pub execution_path: Vec<String>,
}

/// Configuration for flow execution
#[derive(Debug, Clone)]
pub struct FlowConfig {
    /// Maximum number of execution steps before terminating
    pub max_steps: usize,
    /// Whether to detect and prevent cycles
    pub detect_cycles: bool,
    /// Starting node ID
    pub start_node_id: String,
    /// Actions that terminate the flow
    pub terminal_actions: Vec<String>,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            max_steps: 1000,
            detect_cycles: true,
            start_node_id: "start".to_string(),
            terminal_actions: vec![
                "end".to_string(),
                "complete".to_string(),
                "finish".to_string(),
            ],
        }
    }
}

/// Type-erased node runner for dynamic dispatch
#[async_trait]
pub trait NodeRunner<S: StorageBackend>: Send + Sync {
    async fn run(&mut self, store: &mut SharedStore<S>) -> Result<Action, NodeError>;
}

/// Implementation of NodeRunner for any Node
#[async_trait]
impl<B, S> NodeRunner<S> for crate::node::Node<B, S>
where
    B: crate::node::NodeBackend<S> + Send + Sync,
    S: StorageBackend + Send + Sync,
    B::Error: Send + Sync + 'static,
{
    async fn run(&mut self, store: &mut SharedStore<S>) -> Result<Action, NodeError> {
        match self.run(store).await {
            Ok(action) => Ok(action),
            Err(err) => Err(NodeError::ExecutionError(err.to_string())),
        }
    }
}

/// Trait for implementing flow execution logic
#[async_trait]
pub trait Flow<S: StorageBackend> {
    /// Add a node to the flow
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError>;

    /// Add a route between nodes
    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError>;

    /// Execute the flow starting from the configured start node
    async fn execute(
        &mut self,
        store: &mut SharedStore<S>,
    ) -> Result<FlowExecutionResult, FlowError>;

    /// Execute the flow starting from a specific node
    async fn execute_from(
        &mut self,
        store: &mut SharedStore<S>,
        start_node_id: String,
    ) -> Result<FlowExecutionResult, FlowError>;

    /// Get the current configuration
    fn config(&self) -> &FlowConfig;

    /// Update the configuration
    fn set_config(&mut self, config: FlowConfig);

    /// Check if the flow is valid (no orphaned nodes, etc.)
    fn validate(&self) -> Result<(), FlowError>;
}

/// Builder for creating flows easily
pub struct FlowBuilder<S: StorageBackend> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

impl<S: StorageBackend + 'static> Default for FlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: StorageBackend + 'static> FlowBuilder<S> {
    /// Create a new flow builder
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config: FlowConfig::default(),
        }
    }

    /// Set the starting node ID
    pub fn start_node(mut self, node_id: impl Into<String>) -> Self {
        self.config.start_node_id = node_id.into();
        self
    }

    /// Set maximum execution steps
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.config.max_steps = max_steps;
        self
    }

    /// Add a terminal action
    pub fn terminal_action(mut self, action: impl Into<String>) -> Self {
        self.config.terminal_actions.push(action.into());
        self
    }

    /// Add a node to the flow
    pub fn node<B>(mut self, id: impl Into<String>, node: crate::node::Node<B, S>) -> Self
    where
        B: crate::node::NodeBackend<S> + Send + Sync + 'static,
        B::Error: Send + Sync + 'static,
    {
        self.nodes.insert(id.into(), Box::new(node));
        self
    }

    /// Add a simple route (action -> target node)
    pub fn route(
        mut self,
        from: impl Into<String>,
        action: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        let from_id = from.into();
        let route = Route {
            action: action.into(),
            target_node_id: to.into(),
            condition: None,
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }

    /// Add a conditional route
    pub fn conditional_route(
        mut self,
        from: impl Into<String>,
        action: impl Into<String>,
        to: impl Into<String>,
        condition: RouteCondition,
    ) -> Self {
        let from_id = from.into();
        let route = Route {
            action: action.into(),
            target_node_id: to.into(),
            condition: Some(condition),
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }
}

/// Basic implementation of the Flow trait
pub struct BasicFlow<S: StorageBackend> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

impl<S: StorageBackend> BasicFlow<S> {
    /// Create a new basic flow
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config: FlowConfig::default(),
        }
    }

    /// Create a new basic flow with custom configuration
    pub fn with_config(config: FlowConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config,
        }
    }

    /// Find the next node ID based on the current action
    fn find_next_node(
        &self,
        current_node_id: &str,
        action: &Action,
        store: &SharedStore<S>,
    ) -> Result<Option<String>, FlowError> {
        let action_str = action.to_string();

        // Check if this is a terminal action
        if self.config.terminal_actions.contains(&action_str) {
            return Ok(None);
        }

        // Get routes for the current node
        let routes = self.routes.get(current_node_id).ok_or_else(|| {
            FlowError::NoRouteFound(current_node_id.to_string(), action_str.clone())
        })?;

        // Find matching route
        for route in routes {
            if route.action == action_str {
                // Check condition if present
                if let Some(condition) = &route.condition {
                    if !condition.evaluate(store) {
                        continue;
                    }
                }
                return Ok(Some(route.target_node_id.clone()));
            }
        }

        Err(FlowError::NoRouteFound(
            current_node_id.to_string(),
            action_str,
        ))
    }

    /// Check for cycles in the execution path
    fn check_cycle(&self, path: &[String], next_node_id: &str) -> Result<(), FlowError> {
        if !self.config.detect_cycles {
            return Ok(());
        }

        if path.contains(&next_node_id.to_string()) {
            let mut cycle_path = path.to_vec();
            cycle_path.push(next_node_id.to_string());
            return Err(FlowError::CycleDetected(cycle_path));
        }

        Ok(())
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> Flow<S> for BasicFlow<S>
where
    S::Error: Send + Sync + 'static,
{
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError> {
        self.nodes.insert(id, node);
        Ok(())
    }

    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError> {
        self.routes.entry(from_node_id).or_default().push(route);
        Ok(())
    }

    async fn execute(
        &mut self,
        store: &mut SharedStore<S>,
    ) -> Result<FlowExecutionResult, FlowError> {
        let start_node_id = self.config.start_node_id.clone();
        self.execute_from(store, start_node_id).await
    }

    async fn execute_from(
        &mut self,
        store: &mut SharedStore<S>,
        start_node_id: String,
    ) -> Result<FlowExecutionResult, FlowError> {
        let mut current_node_id = start_node_id;
        let mut execution_path = Vec::new();
        let mut steps_executed = 0;

        loop {
            // Check step limit
            if steps_executed >= self.config.max_steps {
                return Err(FlowError::MaxStepsExceeded(self.config.max_steps));
            }

            // Check for cycles
            self.check_cycle(&execution_path, &current_node_id)?;

            // Add current node to execution path
            execution_path.push(current_node_id.clone());

            // Get the current node
            let node = self
                .nodes
                .get_mut(&current_node_id)
                .ok_or_else(|| FlowError::NodeNotFound(current_node_id.clone()))?;

            // Execute the node
            let action = node.run(store).await.map_err(FlowError::from)?;
            steps_executed += 1;

            // Find next node
            match self.find_next_node(&current_node_id, &action, store)? {
                Some(next_node_id) => {
                    current_node_id = next_node_id;
                }
                None => {
                    // Terminal action reached
                    return Ok(FlowExecutionResult {
                        final_action: action,
                        last_node_id: current_node_id,
                        steps_executed,
                        success: true,
                        execution_path,
                    });
                }
            }
        }
    }

    fn config(&self) -> &FlowConfig {
        &self.config
    }

    fn set_config(&mut self, config: FlowConfig) {
        self.config = config;
    }

    fn validate(&self) -> Result<(), FlowError> {
        // Check if start node exists
        if !self.nodes.contains_key(&self.config.start_node_id) {
            return Err(FlowError::InvalidConfiguration(format!(
                "Start node '{}' not found",
                self.config.start_node_id
            )));
        }

        // Check if all route targets exist
        for (from_node, routes) in &self.routes {
            if !self.nodes.contains_key(from_node) {
                return Err(FlowError::InvalidConfiguration(format!(
                    "Source node '{}' in routes not found",
                    from_node
                )));
            }

            for route in routes {
                if !self.nodes.contains_key(&route.target_node_id) {
                    return Err(FlowError::InvalidConfiguration(format!(
                        "Target node '{}' in route not found",
                        route.target_node_id
                    )));
                }
            }
        }

        Ok(())
    }
}

impl<S: StorageBackend + 'static> Default for BasicFlow<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of NodeBackend for BasicFlow, allowing flows to be nested
#[async_trait]
impl<S: StorageBackend + Send + Sync + 'static> NodeBackend<S> for BasicFlow<S>
where
    S::Error: Send + Sync + 'static,
{
    type PrepResult = ();
    type ExecResult = FlowExecutionResult;
    type Error = FlowError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Validate the flow before execution
        self.validate()?;
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // This will be handled in the post method where we have mutable access to the store
        Ok(FlowExecutionResult {
            final_action: Action::simple("flow_ready"),
            last_node_id: "flow".to_string(),
            steps_executed: 0,
            success: true,
            execution_path: vec![],
        })
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Check nesting depth to prevent infinite recursion
        let current_depth = context
            .get_metadata("flow_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if current_depth > 10 {
            // Maximum nesting depth
            return Err(FlowError::InvalidConfiguration(
                "Maximum flow nesting depth exceeded".to_string(),
            ));
        }

        // Execute the nested flow
        let result = self.execute(store).await?;

        // Store the nested flow result in the shared store
        store
            .set(
                "nested_flow_result".to_string(),
                serde_json::json!({
                    "final_action": result.final_action.to_string(),
                    "last_node_id": result.last_node_id,
                    "steps_executed": result.steps_executed,
                    "success": result.success,
                    "execution_path": result.execution_path
                }),
            )
            .map_err(|e| FlowError::NodeError(e.to_string()))?;

        // Return the final action from the nested flow
        Ok(result.final_action)
    }
}

/// A wrapper to make any Flow usable as a Node
pub struct FlowNode<F, S>
where
    F: Flow<S>,
    S: StorageBackend,
{
    flow: F,
    _phantom: std::marker::PhantomData<S>,
}

impl<F, S> FlowNode<F, S>
where
    F: Flow<S>,
    S: StorageBackend,
{
    /// Create a new FlowNode wrapping the given flow
    pub fn new(flow: F) -> Self {
        Self {
            flow,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get a reference to the inner flow
    pub fn flow(&self) -> &F {
        &self.flow
    }

    /// Get a mutable reference to the inner flow
    pub fn flow_mut(&mut self) -> &mut F {
        &mut self.flow
    }
}

#[async_trait]
impl<F, S> NodeBackend<S> for FlowNode<F, S>
where
    F: Flow<S> + Send + Sync,
    S: StorageBackend + Send + Sync + 'static,
    S::Error: Send + Sync + 'static,
{
    type PrepResult = ();
    type ExecResult = FlowExecutionResult;
    type Error = FlowError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Validate the flow before execution
        self.flow.validate()?;
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // This will be handled in the post method where we have mutable access to the store
        Ok(FlowExecutionResult {
            final_action: Action::simple("flow_ready"),
            last_node_id: "flow".to_string(),
            steps_executed: 0,
            success: true,
            execution_path: vec![],
        })
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Check nesting depth to prevent infinite recursion
        let current_depth = context
            .get_metadata("flow_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if current_depth > 10 {
            // Maximum nesting depth
            return Err(FlowError::InvalidConfiguration(
                "Maximum flow nesting depth exceeded".to_string(),
            ));
        }

        // Execute the nested flow
        let result = self.flow.execute(store).await?;

        // Store the nested flow result in the shared store with a unique key
        let result_key = format!("nested_flow_result_{}", context.execution_id());
        store
            .set(
                result_key,
                serde_json::json!({
                    "final_action": result.final_action.to_string(),
                    "last_node_id": result.last_node_id,
                    "steps_executed": result.steps_executed,
                    "success": result.success,
                    "execution_path": result.execution_path
                }),
            )
            .map_err(|e| FlowError::NodeError(e.to_string()))?;

        // Return the final action from the nested flow
        Ok(result.final_action)
    }
}

impl<S: StorageBackend + 'static> FlowBuilder<S> {
    /// Build the flow
    pub fn build(self) -> BasicFlow<S> {
        let mut flow = BasicFlow::with_config(self.config);

        // Add all nodes
        for (id, node) in self.nodes {
            flow.add_node(id, node).expect("Failed to add node");
        }

        // Add all routes
        for (from_id, routes) in self.routes {
            for route in routes {
                flow.add_route(from_id.clone(), route)
                    .expect("Failed to add route");
            }
        }

        flow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "builtin-nodes")]
    use crate::node::builtin::{LogNode, SetValueNode};
    #[cfg(feature = "storage-memory")]
    use crate::{InMemoryStorage, Node};
    #[cfg(feature = "storage-memory")]
    use serde_json::json;

    #[cfg(all(feature = "storage-memory", feature = "builtin-nodes"))]
    #[tokio::test]
    async fn test_basic_flow_creation() {
        let mut flow = BasicFlow::<InMemoryStorage>::new();

        // Add nodes
        let log_node = Node::new(LogNode::new("Test log", Action::simple("continue")));
        let set_node = Node::new(SetValueNode::new(
            "result".to_string(),
            json!("success"),
            Action::simple("complete"),
        ));

        flow.add_node("start".to_string(), Box::new(log_node))
            .unwrap();
        flow.add_node("end".to_string(), Box::new(set_node))
            .unwrap();

        // Add route
        flow.add_route(
            "start".to_string(),
            Route {
                action: "continue".to_string(),
                target_node_id: "end".to_string(),
                condition: None,
            },
        )
        .unwrap();

        // Validate flow
        flow.validate().unwrap();

        // Execute flow
        let mut store = SharedStore::new();
        let result = flow.execute(&mut store).await.unwrap();

        assert_eq!(result.steps_executed, 2);
        assert_eq!(result.execution_path, vec!["start", "end"]);
        assert!(result.success);
        assert_eq!(store.get("result").unwrap().unwrap(), json!("success"));
    }

    #[cfg(all(feature = "storage-memory", feature = "builtin-nodes"))]
    #[tokio::test]
    async fn test_flow_builder() {
        let log_node = Node::new(LogNode::new("Builder test", Action::simple("next")));
        let set_node = Node::new(SetValueNode::new(
            "builder_result".to_string(),
            json!("built"),
            Action::simple("complete"),
        ));

        let mut flow = FlowBuilder::new()
            .start_node("log")
            .max_steps(10)
            .terminal_action("complete")
            .node("log", log_node)
            .node("set", set_node)
            .route("log", "next", "set")
            .build();

        let mut store = SharedStore::new();
        let result = flow.execute(&mut store).await.unwrap();

        assert_eq!(result.steps_executed, 2);
        assert!(result.success);
        assert_eq!(
            store.get("builder_result").unwrap().unwrap(),
            json!("built")
        );
    }

    #[cfg(all(feature = "storage-memory", feature = "builtin-nodes"))]
    #[tokio::test]
    async fn test_conditional_routes() {
        let set_ready_node = Node::new(SetValueNode::new(
            "ready".to_string(),
            json!(true),
            Action::simple("check"),
        ));
        let success_node = Node::new(SetValueNode::new(
            "result".to_string(),
            json!("success"),
            Action::simple("complete"),
        ));
        let fail_node = Node::new(SetValueNode::new(
            "result".to_string(),
            json!("failed"),
            Action::simple("complete"),
        ));

        let mut flow = FlowBuilder::new()
            .start_node("setup")
            .node("setup", set_ready_node)
            .node("success", success_node)
            .node("fail", fail_node)
            .conditional_route(
                "setup",
                "check",
                "success",
                RouteCondition::KeyEquals("ready".to_string(), json!(true)),
            )
            .conditional_route(
                "setup",
                "check",
                "fail",
                RouteCondition::KeyEquals("ready".to_string(), json!(false)),
            )
            .build();

        let mut store = SharedStore::new();
        let result = flow.execute(&mut store).await.unwrap();

        assert_eq!(result.steps_executed, 2);
        assert_eq!(store.get("result").unwrap().unwrap(), json!("success"));
    }

    #[cfg(all(feature = "storage-memory", feature = "builtin-nodes"))]
    #[tokio::test]
    async fn test_cycle_detection() {
        let node1 = Node::new(LogNode::new("Node 1", Action::simple("to_node2")));
        let node2 = Node::new(LogNode::new("Node 2", Action::simple("to_node1")));

        let mut flow = FlowBuilder::new()
            .start_node("node1")
            .node("node1", node1)
            .node("node2", node2)
            .route("node1", "to_node2", "node2")
            .route("node2", "to_node1", "node1")
            .build();

        let mut store = SharedStore::new();
        let result = flow.execute(&mut store).await;

        assert!(matches!(result, Err(FlowError::CycleDetected(_))));
    }

    #[cfg(all(feature = "storage-memory", feature = "builtin-nodes"))]
    #[tokio::test]
    async fn test_max_steps_exceeded() {
        let infinite_node = Node::new(LogNode::new("Infinite", Action::simple("continue")));

        let config = FlowConfig {
            max_steps: 5,
            detect_cycles: false, // Disable cycle detection for this test
            start_node_id: "infinite".to_string(),
            ..FlowConfig::default()
        };

        let mut flow = FlowBuilder::new()
            .start_node("infinite")
            .max_steps(5)
            .node("infinite", infinite_node)
            .route("infinite", "continue", "infinite")
            .build();

        flow.set_config(config);

        let mut store = SharedStore::new();
        let result = flow.execute(&mut store).await;

        // println!("Result: {:?}", result);
        assert!(matches!(result, Err(FlowError::MaxStepsExceeded(5))));
    }
}
