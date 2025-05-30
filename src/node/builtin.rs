use crate::node::{NodeBackend, ExecutionContext, NodeError};
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
    
    async fn prep(&mut self, _store: &SharedStore<S>, context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        Ok(format!("Execution {}: {}", context.execution_id, self.message))
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        println!("{}", prep_result);
        Ok(prep_result)
    }
    
    async fn post(&mut self, _store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  _exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
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

/// A node that makes API requests to OpenAI or compatible APIs
pub struct ApiRequestNode {
    /// Configuration for the API
    config: ApiConfig,
    /// Key to read the prompt from shared store
    prompt_key: String,
    /// Key to store the response in shared store
    output_key: String,
    /// Action to take after successful execution
    action: Action,
    /// Maximum number of retries
    max_retries: usize,
    /// Delay between retries
    retry_delay: Duration,
    /// System message (optional)
    system_message: Option<String>,
}

impl ApiRequestNode {
    /// Create a new API request node
    pub fn new<S1: Into<String>, S2: Into<String>>(
        config: ApiConfig,
        prompt_key: S1,
        output_key: S2,
        action: Action,
    ) -> Self {
        Self {
            config,
            prompt_key: prompt_key.into(),
            output_key: output_key.into(),
            action,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            system_message: None,
        }
    }
    
    /// Set system message
    pub fn with_system_message<S: Into<String>>(mut self, system_message: S) -> Self {
        self.system_message = Some(system_message.into());
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
    
    /// Update API configuration
    pub fn with_config(mut self, config: ApiConfig) -> Self {
        self.config = config;
        self
    }
}

/// Prepared data for API request
#[derive(Debug, Clone)]
pub struct ApiRequestPrep {
    pub prompt: String,
    pub system_message: Option<String>,
    pub config: ApiConfig,
}

/// Result from API execution
#[derive(Debug)]
pub struct ApiRequestResult {
    pub response: String,
    pub usage: Option<serde_json::Value>,
    pub model: String,
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for ApiRequestNode {
    type PrepResult = ApiRequestPrep;
    type ExecResult = ApiRequestResult;
    type Error = NodeError;
    
    async fn prep(&mut self, store: &SharedStore<S>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        // Get prompt from store
        let prompt = store.get(&self.prompt_key)
            .map_err(|e| NodeError::StorageError(e.to_string()))?
            .ok_or_else(|| NodeError::ValidationError(format!("Prompt not found at key: {}", self.prompt_key)))?;
        
        let prompt_str = match prompt {
            Value::String(s) => s,
            other => other.to_string(),
        };
        
        Ok(ApiRequestPrep {
            prompt: prompt_str,
            system_message: self.system_message.clone(),
            config: self.config.clone(),
        })
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        use async_openai::{Client, config::OpenAIConfig};
        use async_openai::types::{
            CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, 
            ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        };
        
        // Create OpenAI client
        let mut config = OpenAIConfig::new().with_api_key(&prep_result.config.api_key);
        
        if let Some(base_url) = &prep_result.config.base_url {
            config = config.with_api_base(base_url);
        }
        
        if let Some(org_id) = &prep_result.config.org_id {
            config = config.with_org_id(org_id);
        }
        
        let client = Client::with_config(config);
        
        // Prepare messages
        let mut messages = Vec::new();
        
        // Add system message if provided
        if let Some(system_msg) = &prep_result.system_message {
            let system_message = ChatCompletionRequestSystemMessageArgs::default()
                .content(system_msg)
                .build()
                .map_err(|e| NodeError::ExecutionError(format!("Failed to build system message: {}", e)))?;
            messages.push(ChatCompletionRequestMessage::System(system_message));
        }
        
        // Add user message
        let user_message = ChatCompletionRequestUserMessageArgs::default()
            .content(prep_result.prompt.clone())
            .build()
            .map_err(|e| NodeError::ExecutionError(format!("Failed to build user message: {}", e)))?;
        messages.push(ChatCompletionRequestMessage::User(user_message));
        
        // Create request
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&prep_result.config.model)
            .messages(messages);
        
        if let Some(max_tokens) = prep_result.config.max_tokens {
            request_builder.max_tokens(max_tokens);
        }
        
        if let Some(temperature) = prep_result.config.temperature {
            request_builder.temperature(temperature);
        }
        
        let request = request_builder
            .build()
            .map_err(|e| NodeError::ExecutionError(format!("Failed to build request: {}", e)))?;
        
        // Make API call
        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| NodeError::ExecutionError(format!("API request failed: {}", e)))?;
        
        // Extract response
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| NodeError::ExecutionError("No response content received".to_string()))?;
        
        let usage = response.usage.map(|u| serde_json::to_value(u).unwrap_or_default());
        
        Ok(ApiRequestResult {
            response: content.clone(),
            usage,
            model: response.model,
        })
    }
    
    async fn post(&mut self, store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
        // Store the response
        match store.set(self.output_key.clone(), serde_json::Value::String(exec_result.response)) {
            Ok(_) => {
                // Optionally store usage information
                if let Some(usage) = exec_result.usage {
                    let usage_key = format!("{}_usage", self.output_key);
                    let _ = store.set(usage_key, usage);
                }
                
                // Store model information
                let model_key = format!("{}_model", self.output_key);
                let _ = store.set(model_key, serde_json::Value::String(exec_result.model));
                
                Ok(self.action.clone())
            },
            Err(e) => Err(NodeError::StorageError(e.to_string())),
        }
    }
    
    async fn exec_fallback(&mut self, _prep_result: Self::PrepResult, error: Self::Error, 
                          _context: &ExecutionContext) -> Result<Self::ExecResult, Self::Error> {
        // Provide a fallback response instead of failing completely
        Ok(ApiRequestResult {
            response: format!("API request failed: {}. Using fallback response.", error),
            usage: None,
            model: "fallback".to_string(),
        })
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
    
    async fn prep(&mut self, _store: &SharedStore<S>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        Ok((self.key.clone(), self.value.clone()))
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        // Simulate some processing
        Ok(prep_result)
    }
    
    async fn post(&mut self, store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
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
        action: Action
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
    
    async fn prep(&mut self, store: &SharedStore<S>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        match store.get(&self.key) {
            Ok(value) => Ok(value),
            Err(e) => Err(NodeError::StorageError(e.to_string())),
        }
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        Ok((self.transform)(prep_result))
    }
    
    async fn post(&mut self, store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
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
    
    async fn prep(&mut self, store: &SharedStore<S>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        Ok((self.condition)(store))
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        Ok(prep_result)
    }
    
    async fn post(&mut self, _store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
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
    
    async fn prep(&mut self, _store: &SharedStore<S>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        Ok(self.duration)
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        tokio::time::sleep(prep_result).await;
        Ok(())
    }
    
    async fn post(&mut self, _store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  _exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
        Ok(self.action.clone())
    }
    
    fn name(&self) -> &str {
        "DelayNode"
    }
    
    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A mock LLM node for testing and examples
pub struct MockLlmNode {
    prompt_key: String,
    output_key: String,
    mock_response: String,
    action: Action,
    max_retries: usize,
    retry_delay: Duration,
    failure_rate: f64, // 0.0 = never fail, 1.0 = always fail
}

impl MockLlmNode {
    /// Create a new mock LLM node
    pub fn new<S1, S2, S3>(
        prompt_key: S1, 
        output_key: S2, 
        mock_response: S3, 
        action: Action
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
    
    async fn prep(&mut self, store: &SharedStore<S>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        let value = match store.get(&self.prompt_key) {
            Ok(value) => value,
            Err(e) => return Err(NodeError::StorageError(e.to_string())),
        };
        
        let prompt = value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| NodeError::ValidationError(format!("Prompt not found at key: {}", self.prompt_key)))?;
        Ok(prompt)
    }
    
    async fn exec(&mut self, prompt: Self::PrepResult, context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        // Simulate API call delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Simulate random failures for testing
        if self.failure_rate > 0.0 && rand::random::<f64>() < self.failure_rate {
            return Err(NodeError::ExecutionError(format!("Mock LLM API failure (retry {})", context.current_retry)));
        }
        
        // Generate mock response
        let response = format!("{} (processed prompt: '{}')", self.mock_response, prompt);
        Ok(response)
    }
    
    async fn post(&mut self, store: &mut SharedStore<S>, _prep_result: Self::PrepResult, 
                  exec_result: Self::ExecResult, _context: &ExecutionContext) 
        -> Result<Action, Self::Error> {
        match store.set(self.output_key.clone(), serde_json::Value::String(exec_result)) {
            Ok(_) => Ok(self.action.clone()),
            Err(e) => Err(NodeError::StorageError(e.to_string())),
        }
    }
    
    async fn exec_fallback(&mut self, _prep_result: Self::PrepResult, error: Self::Error, 
                          _context: &ExecutionContext) -> Result<Self::ExecResult, Self::Error> {
        // Provide a fallback response instead of failing
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