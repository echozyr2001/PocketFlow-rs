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
    use async_openai::{
        Client,
        config::OpenAIConfig,
        types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    };
    use async_trait::async_trait;
    use serde_json::Value;
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
        /// Top-p sampling parameter
        pub top_p: Option<f32>,
        /// Frequency penalty
        pub frequency_penalty: Option<f32>,
        /// Presence penalty
        pub presence_penalty: Option<f32>,
    }

    impl Default for ApiConfig {
        fn default() -> Self {
            Self {
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                base_url: None,
                org_id: None,
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: Some(1000),
                temperature: Some(0.7),
                timeout: Some(30),
                top_p: None,
                frequency_penalty: None,
                presence_penalty: None,
            }
        }
    }

    impl ApiConfig {
        /// Create a new ApiConfig with an API key
        pub fn new(api_key: impl Into<String>) -> Self {
            Self {
                api_key: api_key.into(),
                ..Default::default()
            }
        }

        /// Set the model to use
        pub fn with_model(mut self, model: impl Into<String>) -> Self {
            self.model = model.into();
            self
        }

        /// Set the base URL for the API
        pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
            self.base_url = Some(base_url.into());
            self
        }

        /// Set the organization ID
        pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
            self.org_id = Some(org_id.into());
            self
        }

        /// Set maximum tokens for response
        pub fn with_max_tokens(mut self, max_tokens: u16) -> Self {
            self.max_tokens = Some(max_tokens);
            self
        }

        /// Set temperature for response generation
        pub fn with_temperature(mut self, temperature: f32) -> Self {
            self.temperature = Some(temperature);
            self
        }

        /// Set request timeout in seconds
        pub fn with_timeout(mut self, timeout: u64) -> Self {
            self.timeout = Some(timeout);
            self
        }

        /// Set top-p sampling parameter
        pub fn with_top_p(mut self, top_p: f32) -> Self {
            self.top_p = Some(top_p);
            self
        }

        /// Set frequency penalty
        pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
            self.frequency_penalty = Some(frequency_penalty);
            self
        }

        /// Set presence penalty
        pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
            self.presence_penalty = Some(presence_penalty);
            self
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

    /// HTTP-based API request node for LLM interactions using async-openai SDK
    ///
    /// This node makes actual HTTP requests to LLM APIs (OpenAI, etc.)
    /// It supports various configuration options including retries,
    /// custom endpoints, message history, and error handling.
    #[derive(Debug, Clone)]
    pub struct ApiRequestNode {
        /// Configuration for the API
        config: ApiConfig,
        /// Input key for the messages (can be a single prompt or array of messages)
        input_key: String,
        /// Output key for the response
        output_key: String,
        /// Action to execute after successful completion
        action: Action,
        /// Maximum number of retries
        max_retries: usize,
        /// Delay between retries
        retry_delay: Duration,
        /// System message to prepend to conversations
        system_message: Option<String>,
        /// Cached OpenAI client
        client: Option<Client<OpenAIConfig>>,
    }

    impl ApiRequestNode {
        /// Create a new API request node with default configuration
        pub fn new<S: Into<String>>(input_key: S, output_key: S, action: Action) -> Self {
            Self {
                config: ApiConfig::default(),
                input_key: input_key.into(),
                output_key: output_key.into(),
                action,
                max_retries: 3,
                retry_delay: Duration::from_millis(1000),
                system_message: None,
                client: None,
            }
        }

        /// Create a new API request node with custom configuration
        pub fn with_config(mut self, config: ApiConfig) -> Self {
            self.config = config;
            self
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

        /// Set a system message to prepend to conversations
        pub fn with_system_message(mut self, message: impl Into<String>) -> Self {
            self.system_message = Some(message.into());
            self
        }

        /// Update the configuration
        pub fn update_config(mut self, config: ApiConfig) -> Self {
            self.config = config;
            self.client = None; // Reset client to force recreation
            self
        }

        /// Get or create an OpenAI client
        fn get_client(&mut self) -> Result<&Client<OpenAIConfig>, NodeError> {
            if self.client.is_none() {
                let mut config_builder = OpenAIConfig::new().with_api_key(&self.config.api_key);

                if let Some(ref base_url) = self.config.base_url {
                    config_builder = config_builder.with_api_base(base_url);
                }

                if let Some(ref org_id) = self.config.org_id {
                    config_builder = config_builder.with_org_id(org_id);
                }

                self.client = Some(Client::with_config(config_builder));
            }

            Ok(self.client.as_ref().unwrap())
        }

        /// Convert input to messages array
        fn parse_messages(
            &self,
            input: &Value,
        ) -> Result<Vec<ChatCompletionRequestMessage>, NodeError> {
            let mut messages = Vec::new();

            // Add system message if provided
            if let Some(ref system_msg) = self.system_message {
                messages.push(ChatCompletionRequestMessage::System(
                    async_openai::types::ChatCompletionRequestSystemMessage {
                        content: system_msg.clone().into(),
                        name: None,
                    },
                ));
            }

            // Parse input as either a single prompt or array of messages
            match input {
                Value::String(prompt) => {
                    // Single prompt string - create user message
                    messages.push(ChatCompletionRequestMessage::User(
                        async_openai::types::ChatCompletionRequestUserMessage {
                            content: prompt.clone().into(),
                            name: None,
                        },
                    ));
                }
                Value::Array(message_array) => {
                    // Array of message objects
                    for msg_value in message_array {
                        let role =
                            msg_value
                                .get("role")
                                .and_then(|r| r.as_str())
                                .ok_or_else(|| {
                                    NodeError::ValidationError(
                                        "Message must have a 'role' field".to_string(),
                                    )
                                })?;

                        let content = msg_value
                            .get("content")
                            .and_then(|c| c.as_str())
                            .ok_or_else(|| {
                                NodeError::ValidationError(
                                    "Message must have a 'content' field".to_string(),
                                )
                            })?
                            .to_string();

                        match role {
                            "system" => {
                                messages.push(ChatCompletionRequestMessage::System(
                                    async_openai::types::ChatCompletionRequestSystemMessage {
                                        content: content.into(),
                                        name: msg_value
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .map(|s| s.to_string()),
                                    },
                                ));
                            }
                            "user" => {
                                messages.push(ChatCompletionRequestMessage::User(
                                    async_openai::types::ChatCompletionRequestUserMessage {
                                        content: content.into(),
                                        name: msg_value
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .map(|s| s.to_string()),
                                    },
                                ));
                            }
                            "assistant" => {
                                messages.push(ChatCompletionRequestMessage::Assistant(
                                    async_openai::types::ChatCompletionRequestAssistantMessage {
                                        content: Some(content.into()),
                                        name: msg_value
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .map(|s| s.to_string()),
                                        ..Default::default()
                                    },
                                ));
                            }
                            _ => {
                                return Err(NodeError::ValidationError(format!(
                                    "Unsupported message role: {}",
                                    role
                                )));
                            }
                        }
                    }
                }
                _ => {
                    return Err(NodeError::ValidationError(
                        "Input must be a string (prompt) or array of message objects".to_string(),
                    ));
                }
            }

            if messages.is_empty() {
                return Err(NodeError::ValidationError(
                    "No valid messages found in input".to_string(),
                ));
            }

            Ok(messages)
        }

        /// Make the actual API request using async-openai SDK
        async fn make_api_request(
            &mut self,
            messages: Vec<ChatCompletionRequestMessage>,
        ) -> Result<String, NodeError> {
            // Extract config values to avoid borrowing issues
            let model = self.config.model.clone();
            let max_tokens = self.config.max_tokens;
            let temperature = self.config.temperature;
            let top_p = self.config.top_p;
            let frequency_penalty = self.config.frequency_penalty;
            let presence_penalty = self.config.presence_penalty;
            let timeout_secs = self.config.timeout;

            let client = self.get_client()?;

            // Build the request using builder pattern correctly
            let mut request_builder = CreateChatCompletionRequestArgs::default();
            request_builder.model(model);
            request_builder.messages(messages);

            if let Some(max_tokens) = max_tokens {
                request_builder.max_tokens(max_tokens);
            }

            if let Some(temperature) = temperature {
                request_builder.temperature(temperature);
            }

            if let Some(top_p) = top_p {
                request_builder.top_p(top_p);
            }

            if let Some(frequency_penalty) = frequency_penalty {
                request_builder.frequency_penalty(frequency_penalty);
            }

            if let Some(presence_penalty) = presence_penalty {
                request_builder.presence_penalty(presence_penalty);
            }

            let request = request_builder.build().map_err(|e| {
                NodeError::ExecutionError(format!("Failed to build request: {}", e))
            })?;

            // Make the request with timeout
            let response =
                if let Some(timeout_secs) = timeout_secs {
                    tokio::time::timeout(
                        Duration::from_secs(timeout_secs),
                        client.chat().create(request),
                    )
                    .await
                    .map_err(|_| NodeError::ExecutionError("Request timeout".to_string()))?
                    .map_err(|e| NodeError::ExecutionError(format!("API request failed: {}", e)))?
                } else {
                    client.chat().create(request).await.map_err(|e| {
                        NodeError::ExecutionError(format!("API request failed: {}", e))
                    })?
                };

            // Extract the response content
            let content = response
                .choices
                .first()
                .and_then(|choice| choice.message.content.as_ref())
                .ok_or_else(|| {
                    NodeError::ExecutionError("No response content received".to_string())
                })?
                .clone();

            Ok(content)
        }
    }

    #[async_trait]
    impl<S: StorageBackend + Send + Sync> NodeBackend<S> for ApiRequestNode {
        type PrepResult = Vec<ChatCompletionRequestMessage>; // The messages to send
        type ExecResult = String; // The API response
        type Error = NodeError;

        async fn prep(
            &mut self,
            store: &SharedStore<S>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            match store.get(&self.input_key) {
                Ok(Some(value)) => self.parse_messages(&value),
                Ok(None) => Err(NodeError::PrepError(format!(
                    "Input key '{}' not found in store",
                    self.input_key
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
                    "ApiRequestNode retry attempt {} for {} messages",
                    context.current_retry,
                    prep_result.len()
                );
            }

            // Make the actual API request
            self.make_api_request(prep_result).await
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
            Ok(format!(
                "API request failed: {}. Please check your configuration and try again.",
                error
            ))
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
