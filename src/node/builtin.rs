//! Built-in node implementations
//!
//! This module provides pre-built node implementations organized by feature:
//!
//! - Basic nodes (feature: `builtin-nodes`)
//! - LLM nodes (feature: `builtin-llm`)
//!
//! Each feature set can be enabled independently.

// ============================================================================
// BASIC BUILTIN NODES (feature: builtin-nodes)
// ============================================================================

/// Basic utility nodes for common operations
#[cfg(feature = "builtin-nodes")]
pub mod basic {
    use crate::node::{ExecutionContext, NodeBackend, NodeError};
    use crate::{Action, SharedStore, StorageBackend};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::time::Duration;

    /// A simple node that logs messages and passes through
    pub struct LogNode {
        message: String,
        action: Action,
        max_retries: usize,
        retry_delay: Duration,
    }

    impl LogNode {
        /// Create a new log node
        pub fn new<S: Into<String>>(message: S, action: Action) -> Self {
            Self {
                message: message.into(),
                action,
                max_retries: 1,
                retry_delay: Duration::from_secs(0),
            }
        }

        /// Set maximum retries
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
    impl<S: StorageBackend + Send + Sync> NodeBackend<S> for LogNode {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<S>,
            context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok(format!(
                "Execution {}: {}",
                context.execution_id, self.message
            ))
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            println!("{}", prep_result);
            Ok(prep_result)
        }

        async fn post(
            &mut self,
            _store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            _exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(self.action.clone())
        }

        fn name(&self) -> &str {
            "LogNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }

        fn retry_delay(&self) -> Duration {
            self.retry_delay
        }
    }

    /// A node that sets a value in the shared store
    pub struct SetValueNode {
        key: String,
        value: Value,
        action: Action,
        max_retries: usize,
    }

    impl SetValueNode {
        /// Create a new set value node
        pub fn new<S: Into<String>>(key: S, value: Value, action: Action) -> Self {
            Self {
                key: key.into(),
                value,
                action,
                max_retries: 1,
            }
        }

        /// Set maximum retries
        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }
    }

    #[async_trait]
    impl<S: StorageBackend + Send + Sync> NodeBackend<S> for SetValueNode {
        type PrepResult = (String, Value);
        type ExecResult = (String, Value);
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok((self.key.clone(), self.value.clone()))
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok(prep_result)
        }

        async fn post(
            &mut self,
            store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            let (key, value) = exec_result;
            match store.set(key, value) {
                Ok(_) => Ok(self.action.clone()),
                Err(e) => Err(NodeError::StorageError(e.to_string())),
            }
        }

        fn name(&self) -> &str {
            "SetValueNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }
    }

    /// A node that gets a value from the shared store and optionally transforms it
    pub struct GetValueNode<F>
    where
        F: Fn(Option<Value>) -> Value + Send + Sync,
    {
        key: String,
        output_key: String,
        transform: F,
        action: Action,
        max_retries: usize,
    }

    impl<F> GetValueNode<F>
    where
        F: Fn(Option<Value>) -> Value + Send + Sync,
    {
        /// Create a new get value node
        pub fn new<S1: Into<String>, S2: Into<String>>(
            key: S1,
            output_key: S2,
            transform: F,
            action: Action,
        ) -> Self {
            Self {
                key: key.into(),
                output_key: output_key.into(),
                transform,
                action,
                max_retries: 1,
            }
        }

        /// Set maximum retries
        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }
    }

    #[async_trait]
    impl<S, F> NodeBackend<S> for GetValueNode<F>
    where
        S: StorageBackend + Send + Sync,
        F: Fn(Option<Value>) -> Value + Send + Sync,
    {
        type PrepResult = Option<Value>;
        type ExecResult = Value;
        type Error = NodeError;

        async fn prep(
            &mut self,
            store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            match store.get(&self.key) {
                Ok(value) => Ok(value),
                Err(e) => Err(NodeError::StorageError(e.to_string())),
            }
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok((self.transform)(prep_result))
        }

        async fn post(
            &mut self,
            store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            match store.set(self.output_key.clone(), exec_result) {
                Ok(_) => Ok(self.action.clone()),
                Err(e) => Err(NodeError::StorageError(e.to_string())),
            }
        }

        fn name(&self) -> &str {
            "GetValueNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }
    }

    /// A conditional node that chooses actions based on store content
    pub struct ConditionalNode<F, S>
    where
        F: Fn(&SharedStore<S>) -> bool + Send + Sync,
        S: StorageBackend,
    {
        condition: F,
        if_true: Action,
        if_false: Action,
        max_retries: usize,
        _phantom: std::marker::PhantomData<S>,
    }

    impl<F, S> ConditionalNode<F, S>
    where
        F: Fn(&SharedStore<S>) -> bool + Send + Sync,
        S: StorageBackend,
    {
        /// Create a new conditional node
        pub fn new(condition: F, if_true: Action, if_false: Action) -> Self {
            Self {
                condition,
                if_true,
                if_false,
                max_retries: 1,
                _phantom: std::marker::PhantomData,
            }
        }

        /// Set maximum retries
        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }
    }

    #[async_trait]
    impl<S, F> NodeBackend<S> for ConditionalNode<F, S>
    where
        S: StorageBackend + Send + Sync,
        F: Fn(&SharedStore<S>) -> bool + Send + Sync,
    {
        type PrepResult = bool;
        type ExecResult = bool;
        type Error = NodeError;

        async fn prep(
            &mut self,
            store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok((self.condition)(store))
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok(prep_result)
        }

        async fn post(
            &mut self,
            _store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            if exec_result {
                Ok(self.if_true.clone())
            } else {
                Ok(self.if_false.clone())
            }
        }

        fn name(&self) -> &str {
            "ConditionalNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }
    }

    /// A delay node that waits for a specified duration
    pub struct DelayNode {
        duration: Duration,
        action: Action,
        max_retries: usize,
    }

    impl DelayNode {
        /// Create a new delay node
        pub fn new(duration: Duration, action: Action) -> Self {
            Self {
                duration,
                action,
                max_retries: 1,
            }
        }

        /// Set maximum retries
        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }
    }

    #[async_trait]
    impl<S: StorageBackend + Send + Sync> NodeBackend<S> for DelayNode {
        type PrepResult = Duration;
        type ExecResult = ();
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok(self.duration)
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            tokio::time::sleep(prep_result).await;
            Ok(())
        }

        async fn post(
            &mut self,
            _store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            _exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(self.action.clone())
        }

        fn name(&self) -> &str {
            "DelayNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }
    }
}

// ============================================================================
// LLM NODES (feature: builtin-llm)
// ============================================================================

/// LLM-related nodes for AI interactions
#[cfg(feature = "builtin-llm")]
pub mod llm {
    use crate::node::{ExecutionContext, NodeBackend, NodeError};
    use crate::{Action, SharedStore, StorageBackend};
    use async_trait::async_trait;
    use std::time::Duration;

    /// Configuration for API requests
    #[derive(Debug, Clone)]
    pub struct ApiConfig {
        /// API key for authentication
        pub api_key: String,
        /// Base URL for the API (optional, defaults to OpenAI)
        pub base_url: Option<String>,
        /// Organization ID (optional)
        pub org_id: Option<String>,
        /// Model to use for requests
        pub model: String,
        /// Maximum tokens for response
        pub max_tokens: Option<u16>,
        /// Temperature for response generation
        pub temperature: Option<f32>,
        /// Request timeout in seconds
        pub timeout: Option<u64>,
    }

    impl Default for ApiConfig {
        fn default() -> Self {
            Self {
                api_key: String::new(),
                base_url: None,
                org_id: None,
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: Some(1000),
                temperature: Some(0.7),
                timeout: Some(30),
            }
        }
    }

    // LLM nodes implementation will be added here

    /// A mock LLM node for testing and examples
    pub struct MockLlmNode {
        prompt_key: String,
        output_key: String,
        mock_response: String,
        action: Action,
        max_retries: usize,
        retry_delay: Duration,
        failure_rate: f64,
    }

    impl MockLlmNode {
        /// Create a new mock LLM node
        pub fn new<S1, S2, S3>(
            prompt_key: S1,
            output_key: S2,
            mock_response: S3,
            action: Action,
        ) -> Self
        where
            S1: Into<String>,
            S2: Into<String>,
            S3: Into<String>,
        {
            Self {
                prompt_key: prompt_key.into(),
                output_key: output_key.into(),
                mock_response: mock_response.into(),
                action,
                max_retries: 3,
                retry_delay: Duration::from_secs(1),
                failure_rate: 0.0,
            }
        }

        /// Set maximum retries
        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }

        /// Set retry delay
        pub fn with_retry_delay(mut self, delay: Duration) -> Self {
            self.retry_delay = delay;
            self
        }

        /// Set failure rate for testing retry logic
        pub fn with_failure_rate(mut self, rate: f64) -> Self {
            self.failure_rate = rate.clamp(0.0, 1.0);
            self
        }
    }

    #[async_trait]
    impl<S: StorageBackend + Send + Sync> NodeBackend<S> for MockLlmNode {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            let value = match store.get(&self.prompt_key) {
                Ok(value) => value,
                Err(e) => return Err(NodeError::StorageError(e.to_string())),
            };

            let prompt = value
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or_else(|| {
                    NodeError::ValidationError(format!(
                        "Prompt not found at key: {}",
                        self.prompt_key
                    ))
                })?;
            Ok(prompt)
        }

        async fn exec(
            &mut self,
            prompt: Self::PrepResult,
            context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            // Simulate API call delay
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Simulate random failures for testing
            if self.failure_rate > 0.0 && rand::random::<f64>() < self.failure_rate {
                return Err(NodeError::ExecutionError(format!(
                    "Mock LLM API failure (retry {})",
                    context.current_retry
                )));
            }

            // Generate mock response
            let response = format!("{} (processed prompt: '{}')", self.mock_response, prompt);
            Ok(response)
        }

        async fn post(
            &mut self,
            store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            match store.set(
                self.output_key.clone(),
                serde_json::Value::String(exec_result),
            ) {
                Ok(_) => Ok(self.action.clone()),
                Err(e) => Err(NodeError::StorageError(e.to_string())),
            }
        }

        async fn exec_fallback(
            &mut self,
            _prep_result: Self::PrepResult,
            error: Self::Error,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok(format!("Fallback response due to error: {}", error))
        }

        fn name(&self) -> &str {
            "MockLlmNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }

        fn retry_delay(&self) -> Duration {
            self.retry_delay
        }
    }

    /// HTTP-based API request node for LLM interactions
    ///
    /// This node makes actual HTTP requests to LLM APIs (OpenAI, etc.)
    /// It supports various configuration options including retries,
    /// custom endpoints, and error handling.
    #[derive(Debug, Clone)]
    pub struct ApiRequestNode {
        /// Configuration for the API
        config: ApiConfig,
        /// Input key for the prompt
        input_key: String,
        /// Output key for the response
        output_key: String,
        /// Action to execute after successful completion
        action: Action,
        /// Maximum number of retries
        max_retries: usize,
        /// Delay between retries
        retry_delay: Duration,
    }

    impl ApiRequestNode {
        /// Create a new API request node
        pub fn new<S: Into<String>>(
            config: ApiConfig,
            input_key: S,
            output_key: S,
            action: Action,
        ) -> Self {
            Self {
                config,
                input_key: input_key.into(),
                output_key: output_key.into(),
                action,
                max_retries: 3,
                retry_delay: Duration::from_millis(1000),
            }
        }

        /// Set maximum retries
        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }

        /// Set retry delay
        pub fn with_retry_delay(mut self, delay: Duration) -> Self {
            self.retry_delay = delay;
            self
        }

        /// Make the actual HTTP request to the LLM API
        async fn make_api_request(&self, prompt: &str) -> Result<String, NodeError> {
            let client = reqwest::Client::new();
            
            // Use OpenAI API format by default
            let url = self.config.base_url.as_deref()
                .unwrap_or("https://api.openai.com/v1/chat/completions");

            let request_body = serde_json::json!({
                "model": self.config.model,
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "max_tokens": self.config.max_tokens,
                "temperature": self.config.temperature
            });

            let response = client
                .post(url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| NodeError::ExecutionError(format!("API request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(NodeError::ExecutionError(format!(
                    "API returned error {}: {}", status, error_text
                )));
            }

            let response_json: serde_json::Value = response
                .json()
                .await
                .map_err(|e| NodeError::ExecutionError(format!("Failed to parse response: {}", e)))?;

            // Extract the assistant's response from OpenAI format
            let content = response_json
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .ok_or_else(|| NodeError::ExecutionError("Invalid API response format".to_string()))?;

            Ok(content.to_string())
        }
    }

    #[async_trait]
    impl<S: StorageBackend + Send + Sync> NodeBackend<S> for ApiRequestNode {
        type PrepResult = String; // The prompt to send
        type ExecResult = String; // The API response
        type Error = NodeError;

        async fn prep(
            &mut self,
            store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            match store.get(&self.input_key) {
                Ok(Some(value)) => {
                    value.as_str()
                        .ok_or_else(|| NodeError::PrepError(format!(
                            "Input '{}' is not a string", self.input_key
                        )))
                        .map(|s| s.to_string())
                }
                Ok(None) => Err(NodeError::PrepError(format!(
                    "Input key '{}' not found in store", self.input_key
                ))),
                Err(e) => Err(NodeError::StorageError(e.to_string())),
            }
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            // Check if this is a retry and log it
            if context.current_retry > 0 {
                eprintln!(
                    "ApiRequestNode retry attempt {} for prompt: '{}'",
                    context.current_retry, prep_result
                );
            }

            // Make the actual API request
            self.make_api_request(&prep_result).await
        }

        async fn post(
            &mut self,
            store: &mut SharedStore<S>,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            match store.set(
                self.output_key.clone(),
                serde_json::Value::String(exec_result),
            ) {
                Ok(_) => Ok(self.action.clone()),
                Err(e) => Err(NodeError::StorageError(e.to_string())),
            }
        }

        async fn exec_fallback(
            &mut self,
            _prep_result: Self::PrepResult,
            error: Self::Error,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            // For API failures, return a user-friendly error message
            Ok(format!("API request failed: {}. Please check your configuration and try again.", error))
        }

        fn name(&self) -> &str {
            "ApiRequestNode"
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }

        fn retry_delay(&self) -> Duration {
            self.retry_delay
        }
    }
}

// ============================================================================
// RE-EXPORTS FOR CONVENIENCE
// ============================================================================

// Re-export basic nodes
#[cfg(feature = "builtin-nodes")]
pub use basic::{ConditionalNode, DelayNode, GetValueNode, LogNode, SetValueNode};

// Re-export LLM components
#[cfg(feature = "builtin-llm")]
pub use llm::{ApiConfig, ApiRequestNode, MockLlmNode};
