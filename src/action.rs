//! # Action and Condition System
//!
//! This module provides the action system for PocketFlow, which controls how execution flows
//! between nodes in a workflow graph. Actions serve as both edge labels and execution directives.
//!
//! ## Core Concepts
//!
//! ### Actions
//! Actions represent transitions between nodes and can be:
//! - **Simple**: Basic string-based actions for backward compatibility
//! - **Parameterized**: Actions with key-value parameters for context passing
//! - **Conditional**: Actions that evaluate conditions before execution
//! - **Multiple**: Collections of actions for parallel execution or choice points
//! - **Prioritized**: Actions with explicit priority ordering
//! - **WithMetadata**: Actions carrying additional execution metadata
//!
//! ### Conditions
//! Conditions enable dynamic routing based on shared store state:
//! - **KeyExists**: Check if a key is present in the shared store
//! - **KeyEquals**: Compare a key's value to an expected value
//! - **NumericCompare**: Perform numeric comparisons with various operators
//! - **Expression**: String-based custom expressions (for future evaluation engines)
//! - **Logical Operators**: AND, OR, NOT for complex condition composition
//!
//! ## Examples
//!
//! ```rust
//! use pocketflow_rs::prelude::*;
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! // Simple action
//! let continue_action = Action::simple("continue");
//!
//! // Parameterized action
//! let mut params = HashMap::new();
//! params.insert("temperature".to_string(), json!(0.7));
//! let llm_action = Action::with_params("llm_call", params);
//!
//! // Conditional action with logical operators
//! let ready_condition = ActionCondition::key_equals("status", json!("ready"));
//! let valid_condition = ActionCondition::key_exists("user_input");
//! let combined = ActionCondition::and(vec![ready_condition, valid_condition]);
//!
//! let conditional_action = Action::conditional(
//!     combined,
//!     Action::simple("process"),
//!     Action::simple("wait")
//! );
//!
//! // Using the builder pattern
//! let complex_action = ActionBuilder::new("api_call")
//!     .with_param("endpoint", json!("/generate"))
//!     .with_param("method", json!("POST"))
//!     .with_priority(10)
//!     .build();
//! ```
//!
//! ## Design Principles
//!
//! 1. **Composability**: Actions and conditions can be nested and combined arbitrarily
//! 2. **Backward Compatibility**: Simple string actions work in all contexts
//! 3. **Type Safety**: Strong typing with serialization support
//! 4. **Expressiveness**: Rich condition system for complex routing logic
//! 5. **Performance**: Efficient evaluation with minimal allocations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Represents an action that controls flow between nodes in PocketFlow workflows.
/// Actions can be simple labels, parameterized, conditional, or composite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Simple string-based action (backward compatible)
    Simple(String),

    /// Action with parameters
    Parameterized {
        name: String,
        params: HashMap<String, Value>,
    },

    /// Conditional action that evaluates based on context
    Conditional {
        condition: ActionCondition,
        if_true: Box<Action>,
        if_false: Box<Action>,
    },

    /// Multiple actions that can be taken (for parallel execution or choices)
    Multiple(Vec<Action>),

    /// Action with priority (higher numbers = higher priority)
    Prioritized { action: Box<Action>, priority: i32 },

    /// Action with metadata
    WithMetadata {
        action: Box<Action>,
        metadata: HashMap<String, Value>,
    },
}

/// Represents a condition for conditional actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionCondition {
    /// Always true
    Always,

    /// Always false
    Never,

    /// Check if a key exists in the shared store
    KeyExists(String),

    /// Check if a key has a specific value
    KeyEquals(String, Value),

    /// Compare a numeric value
    NumericCompare {
        key: String,
        operator: ComparisonOperator,
        value: f64,
    },

    /// Custom condition with a string expression
    Expression(String),

    /// Logical AND of multiple conditions
    And(Vec<ActionCondition>),

    /// Logical OR of multiple conditions
    Or(Vec<ActionCondition>),

    /// Logical NOT of a condition
    Not(Box<ActionCondition>),
}

/// Comparison operators for numeric conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl Action {
    /// Create a simple action from a string
    pub fn simple<S: Into<String>>(name: S) -> Self {
        Action::Simple(name.into())
    }

    /// Create a parameterized action
    pub fn with_params<S: Into<String>>(name: S, params: HashMap<String, Value>) -> Self {
        Action::Parameterized {
            name: name.into(),
            params,
        }
    }

    /// Create a conditional action
    pub fn conditional(condition: ActionCondition, if_true: Action, if_false: Action) -> Self {
        Action::Conditional {
            condition,
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }

    /// Create a multiple action
    pub fn multiple(actions: Vec<Action>) -> Self {
        Action::Multiple(actions)
    }

    /// Create a prioritized action
    pub fn with_priority(action: Action, priority: i32) -> Self {
        Action::Prioritized {
            action: Box::new(action),
            priority,
        }
    }

    /// Add metadata to an action
    pub fn with_metadata(action: Action, metadata: HashMap<String, Value>) -> Self {
        Action::WithMetadata {
            action: Box::new(action),
            metadata,
        }
    }

    /// Get the primary name/identifier of the action
    pub fn name(&self) -> String {
        match self {
            Action::Simple(name) => name.clone(),
            Action::Parameterized { name, .. } => name.clone(),
            Action::Conditional { if_true, .. } => if_true.name(),
            Action::Multiple(actions) => {
                if let Some(first) = actions.first() {
                    first.name()
                } else {
                    "empty".to_string()
                }
            }
            Action::Prioritized { action, .. } => action.name(),
            Action::WithMetadata { action, .. } => action.name(),
        }
    }

    /// Get parameters if this is a parameterized action
    pub fn params(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::Parameterized { params, .. } => Some(params),
            Action::WithMetadata { action, .. } => action.params(),
            Action::Prioritized { action, .. } => action.params(),
            _ => None,
        }
    }

    /// Get priority if this action has one
    pub fn priority(&self) -> Option<i32> {
        match self {
            Action::Prioritized { priority, .. } => Some(*priority),
            Action::WithMetadata { action, .. } => action.priority(),
            _ => None,
        }
    }

    /// Get metadata if this action has any
    pub fn metadata(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::WithMetadata { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    /// Check if this is a simple action
    pub fn is_simple(&self) -> bool {
        matches!(self, Action::Simple(_))
    }

    /// Check if this action has parameters
    pub fn has_params(&self) -> bool {
        self.params().is_some()
    }

    /// Check if this action is conditional
    pub fn is_conditional(&self) -> bool {
        matches!(self, Action::Conditional { .. })
    }

    /// Check if this action represents multiple actions
    pub fn is_multiple(&self) -> bool {
        matches!(self, Action::Multiple(_))
    }
}

impl ActionCondition {
    /// Create a condition that checks if a key exists
    pub fn key_exists<S: Into<String>>(key: S) -> Self {
        ActionCondition::KeyExists(key.into())
    }

    /// Create a condition that checks if a key equals a value
    pub fn key_equals<S: Into<String>>(key: S, value: Value) -> Self {
        ActionCondition::KeyEquals(key.into(), value)
    }

    /// Create a numeric comparison condition
    pub fn numeric_compare<S: Into<String>>(
        key: S,
        operator: ComparisonOperator,
        value: f64,
    ) -> Self {
        ActionCondition::NumericCompare {
            key: key.into(),
            operator,
            value,
        }
    }

    /// Create an expression-based condition
    pub fn expression<S: Into<String>>(expr: S) -> Self {
        ActionCondition::Expression(expr.into())
    }

    /// Create an AND condition
    pub fn and(conditions: Vec<ActionCondition>) -> Self {
        ActionCondition::And(conditions)
    }

    /// Create an OR condition
    pub fn or(conditions: Vec<ActionCondition>) -> Self {
        ActionCondition::Or(conditions)
    }

    /// Create a NOT condition from another condition
    ///
    /// # Examples
    ///
    /// ```
    /// use pocketflow_rs::action::ActionCondition;
    ///
    /// let condition = ActionCondition::key_equals("status", serde_json::json!("ready"));
    ///
    /// // Using static method
    /// let negated1 = ActionCondition::negate(condition.clone());
    ///
    /// // Using ! operator (trait implementation)  
    /// let negated2 = !condition;
    /// ```
    pub fn negate(condition: ActionCondition) -> Self {
        ActionCondition::Not(Box::new(condition))
    }
}

// 实现标准库的 Not trait
impl std::ops::Not for ActionCondition {
    type Output = Self;

    fn not(self) -> Self::Output {
        ActionCondition::Not(Box::new(self))
    }
}

// Display implementations for better debugging and logging
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Simple(name) => write!(f, "{}", name),
            Action::Parameterized { name, params } => {
                write!(
                    f,
                    "{}({})",
                    name,
                    params
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Action::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                write!(f, "if {} then {} else {}", condition, if_true, if_false)
            }
            Action::Multiple(actions) => {
                write!(
                    f,
                    "[{}]",
                    actions
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Action::Prioritized { action, priority } => {
                write!(f, "{}@{}", action, priority)
            }
            Action::WithMetadata { action, .. } => {
                write!(f, "{}", action)
            }
        }
    }
}

impl fmt::Display for ActionCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionCondition::Always => write!(f, "true"),
            ActionCondition::Never => write!(f, "false"),
            ActionCondition::KeyExists(key) => write!(f, "exists({})", key),
            ActionCondition::KeyEquals(key, value) => write!(f, "{} == {}", key, value),
            ActionCondition::NumericCompare {
                key,
                operator,
                value,
            } => {
                let op_str = match operator {
                    ComparisonOperator::Equal => "==",
                    ComparisonOperator::NotEqual => "!=",
                    ComparisonOperator::GreaterThan => ">",
                    ComparisonOperator::GreaterThanOrEqual => ">=",
                    ComparisonOperator::LessThan => "<",
                    ComparisonOperator::LessThanOrEqual => "<=",
                };
                write!(f, "{} {} {}", key, op_str, value)
            }
            ActionCondition::Expression(expr) => write!(f, "({})", expr),
            ActionCondition::And(conditions) => {
                write!(
                    f,
                    "({})",
                    conditions
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" && ")
                )
            }
            ActionCondition::Or(conditions) => {
                write!(
                    f,
                    "({})",
                    conditions
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" || ")
                )
            }
            ActionCondition::Not(condition) => write!(f, "!({})", condition),
        }
    }
}

// Conversion traits for backward compatibility
impl From<String> for Action {
    fn from(s: String) -> Self {
        Action::Simple(s)
    }
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Action::Simple(s.to_string())
    }
}

impl From<Action> for String {
    fn from(action: Action) -> Self {
        action.name()
    }
}

// Builder pattern for complex actions
pub struct ActionBuilder {
    action: Action,
}

impl ActionBuilder {
    /// Start building a new action
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            action: Action::Simple(name.into()),
        }
    }

    /// Add parameters to the action
    pub fn with_params(mut self, params: HashMap<String, Value>) -> Self {
        match self.action {
            Action::Simple(name) => {
                self.action = Action::Parameterized { name, params };
            }
            _ => {
                // Wrap existing action
                self.action = Action::Parameterized {
                    name: self.action.name(),
                    params,
                };
            }
        }
        self
    }

    /// Add a single parameter
    pub fn with_param<S: Into<String>>(self, key: S, value: Value) -> Self {
        let mut params = HashMap::new();
        params.insert(key.into(), value);
        self.with_params(params)
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.action = Action::Prioritized {
            action: Box::new(self.action),
            priority,
        };
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.action = Action::WithMetadata {
            action: Box::new(self.action),
            metadata,
        };
        self
    }

    /// Build the final action
    pub fn build(self) -> Action {
        self.action
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_action() {
        let action = Action::simple("continue");
        assert_eq!(action.name(), "continue");
        assert!(action.is_simple());
        assert!(!action.has_params());
        assert!(!action.is_conditional());
        assert!(!action.is_multiple());

        // Test display
        assert_eq!(action.to_string(), "continue");
    }

    #[test]
    fn test_parameterized_action() {
        let mut params = HashMap::new();
        params.insert("temperature".to_string(), json!(0.7));
        params.insert("max_tokens".to_string(), json!(100));

        let action = Action::with_params("llm_call", params.clone());
        assert_eq!(action.name(), "llm_call");
        assert!(!action.is_simple());
        assert!(action.has_params());
        assert_eq!(action.params(), Some(&params));

        // Test display
        let display_str = action.to_string();
        assert!(display_str.contains("llm_call"));
        assert!(display_str.contains("temperature=0.7"));
        assert!(display_str.contains("max_tokens=100"));
    }

    #[test]
    fn test_conditional_action() {
        let condition = ActionCondition::key_exists("user_input");
        let if_true = Action::simple("process");
        let if_false = Action::simple("request_input");

        let action = Action::conditional(condition.clone(), if_true.clone(), if_false.clone());
        assert_eq!(action.name(), "process"); // Should return if_true action name
        assert!(action.is_conditional());

        // Test display
        let display_str = action.to_string();
        assert!(display_str.contains("if"));
        assert!(display_str.contains("exists(user_input)"));
        assert!(display_str.contains("process"));
        assert!(display_str.contains("request_input"));
    }

    #[test]
    fn test_multiple_actions() {
        let actions = vec![
            Action::simple("action1"),
            Action::simple("action2"),
            Action::simple("action3"),
        ];

        let multi_action = Action::multiple(actions);
        assert_eq!(multi_action.name(), "action1"); // Should return first action name
        assert!(multi_action.is_multiple());

        // Test display
        let display_str = multi_action.to_string();
        assert!(display_str.contains("["));
        assert!(display_str.contains("action1"));
        assert!(display_str.contains("action2"));
        assert!(display_str.contains("action3"));
    }

    #[test]
    fn test_prioritized_action() {
        let base_action = Action::simple("important");
        let action = Action::with_priority(base_action, 10);

        assert_eq!(action.name(), "important");
        assert_eq!(action.priority(), Some(10));

        // Test display
        assert_eq!(action.to_string(), "important@10");
    }

    #[test]
    fn test_action_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), json!("user"));
        metadata.insert("timestamp".to_string(), json!("2024-01-01"));

        let base_action = Action::simple("process");
        let action = Action::with_metadata(base_action, metadata.clone());

        assert_eq!(action.name(), "process");
        assert_eq!(action.metadata(), Some(&metadata));
    }

    #[test]
    fn test_action_conditions() {
        // Test key exists condition
        let cond1 = ActionCondition::key_exists("test_key");
        assert_eq!(cond1.to_string(), "exists(test_key)");

        // Test key equals condition
        let cond2 = ActionCondition::key_equals("status", json!("ready"));
        assert_eq!(cond2.to_string(), "status == \"ready\"");

        // Test numeric comparison
        let cond3 =
            ActionCondition::numeric_compare("temperature", ComparisonOperator::GreaterThan, 0.5);
        assert_eq!(cond3.to_string(), "temperature > 0.5");

        // Test logical operations
        let cond4 = ActionCondition::and(vec![cond1.clone(), cond2.clone()]);
        assert!(cond4.to_string().contains("&&"));

        let cond5 = ActionCondition::or(vec![cond1.clone(), cond2.clone()]);
        assert!(cond5.to_string().contains("||"));

        let cond6 = ActionCondition::negate(cond1);
        assert!(cond6.to_string().contains("!"));

        // 测试 Not trait
        let cond7 = ActionCondition::key_equals("status", serde_json::json!("ready"));
        let cond8 = !cond7; // 使用 ! 操作符
        assert!(cond8.to_string().contains("!"));
    }

    #[test]
    fn test_action_builder() {
        let mut params = HashMap::new();
        params.insert("model".to_string(), json!("gpt-4"));

        let mut metadata = HashMap::new();
        metadata.insert("created_by".to_string(), json!("system"));

        let action = ActionBuilder::new("complex_action")
            .with_params(params.clone())
            .with_priority(5)
            .with_metadata(metadata.clone())
            .build();

        assert_eq!(action.name(), "complex_action");
        assert_eq!(action.priority(), Some(5));
        assert_eq!(action.metadata(), Some(&metadata));
        // Note: params might be wrapped due to builder implementation
    }

    #[test]
    fn test_action_builder_single_param() {
        let action = ActionBuilder::new("test")
            .with_param("key", json!("value"))
            .build();

        assert!(action.has_params());
        assert_eq!(action.name(), "test");
    }

    #[test]
    fn test_backward_compatibility() {
        // Test conversion from string
        let action1: Action = "continue".into();
        assert_eq!(action1.name(), "continue");
        assert!(action1.is_simple());

        let action2: Action = "retry".to_string().into();
        assert_eq!(action2.name(), "retry");

        // Test conversion to string
        let action3 = Action::simple("finish");
        let name: String = action3.into();
        assert_eq!(name, "finish");
    }

    #[test]
    fn test_serialization() {
        let action = Action::simple("test");
        let json_str = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json_str).unwrap();
        assert_eq!(action, deserialized);

        // Test complex action serialization
        let mut params = HashMap::new();
        params.insert("temp".to_string(), json!(0.7));
        let complex_action = Action::with_params("llm", params);

        let json_str2 = serde_json::to_string(&complex_action).unwrap();
        let deserialized2: Action = serde_json::from_str(&json_str2).unwrap();
        assert_eq!(complex_action, deserialized2);
    }

    #[test]
    fn test_nested_actions() {
        // Test deeply nested action structure
        let base = Action::simple("base");
        let with_priority = Action::with_priority(base, 10);

        let mut metadata = HashMap::new();
        metadata.insert("level".to_string(), json!("nested"));
        let with_metadata = Action::with_metadata(with_priority, metadata);

        assert_eq!(with_metadata.name(), "base");
        assert_eq!(with_metadata.priority(), Some(10));
        assert!(with_metadata.metadata().is_some());
    }
}
