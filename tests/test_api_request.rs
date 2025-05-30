use pocketflow_rs::{
    ApiRequestNode, ApiConfig, Action,
};

#[tokio::test]
async fn test_api_request_node_creation() {
    let config = ApiConfig {
        api_key: "test-key".to_string(),
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.5),
        ..Default::default()
    };
    
    let _api_node = ApiRequestNode::new(
        config,
        "prompt",
        "response",
        Action::simple("continue")
    )
    .with_system_message("You are a helpful assistant")
    .with_retries(2);
    
    // Test that the node was created successfully (compilation test)
    assert!(true);
}

#[tokio::test]
async fn test_api_config_default() {
    let config = ApiConfig::default();
    assert_eq!(config.model, "gpt-3.5-turbo");
    assert_eq!(config.max_tokens, Some(1000));
    assert_eq!(config.temperature, Some(0.7));
    assert_eq!(config.timeout, Some(30));
}