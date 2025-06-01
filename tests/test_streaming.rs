use pocketflow_rs::{
    Action, ExecutionContext, InMemoryStorage, SharedStore,
    node::NodeBackend,
    node::builtin::llm::{ApiConfig, ApiRequestNode},
};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_api_request_node_streaming() {
    // Initialize test environment
    let mut store: SharedStore<InMemoryStorage> = SharedStore::new();
    let execution_context = ExecutionContext::new(0, Duration::from_secs(5));

    // Create API config with streaming enabled
    let api_config = ApiConfig {
        api_key: "test_key".to_string(),
        base_url: None,
        org_id: None,
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        timeout: Some(30),
        stream: true, // Enable streaming
    };

    // Create the API request node
    let mut api_node =
        ApiRequestNode::new("input", "output", Action::simple("next")).with_config(api_config);

    // Set up test data in the store
    let test_data = json!([
        {
            "role": "user",
            "content": "Hello, how are you?"
        }
    ]);

    store.set("input".to_string(), test_data).unwrap();

    println!("Testing streaming API request...");

    // First prep the data
    match api_node.prep(&store, &execution_context).await {
        Ok(messages) => {
            println!(
                "Messages prepared successfully: {} message(s)",
                messages.len()
            );

            // Then execute with the prepared messages (this will fail unless you have a real API key)
            match <ApiRequestNode as NodeBackend<InMemoryStorage>>::exec(
                &mut api_node,
                messages,
                &execution_context,
            )
            .await
            {
                Ok(result) => {
                    println!("Streaming response received: {}", result);
                    assert!(!result.is_empty());
                }
                Err(e) => {
                    // Expected to fail without proper API credentials
                    println!("Expected error (no API key): {}", e);
                    assert!(
                        e.to_string().contains("API")
                            || e.to_string().contains("key")
                            || e.to_string().contains("auth")
                    );
                }
            }
        }
        Err(e) => {
            println!("Prep failed: {}", e);
            // This should succeed since we're just parsing JSON
            panic!("Prep should not fail with valid input data");
        }
    }
}

#[tokio::test]
async fn test_api_request_node_non_streaming() {
    // Initialize test environment
    let mut store: SharedStore<InMemoryStorage> = SharedStore::new();
    let execution_context = ExecutionContext::new(0, Duration::from_secs(5));

    // Create API config with streaming disabled
    let api_config = ApiConfig {
        api_key: "test_key".to_string(),
        base_url: None,
        org_id: None,
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        timeout: Some(30),
        stream: false, // Disable streaming
    };

    // Create the API request node
    let mut api_node =
        ApiRequestNode::new("input", "output", Action::simple("next")).with_config(api_config);

    // Set up test data in the store
    let test_data = json!([
        {
            "role": "user",
            "content": "Hello, how are you?"
        }
    ]);

    store.set("input".to_string(), test_data).unwrap();

    println!("Testing non-streaming API request...");

    // First prep the data
    match api_node.prep(&store, &execution_context).await {
        Ok(messages) => {
            println!(
                "Messages prepared successfully: {} message(s)",
                messages.len()
            );

            // Then execute with the prepared messages (this will fail unless you have a real API key)
            match <ApiRequestNode as NodeBackend<InMemoryStorage>>::exec(
                &mut api_node,
                messages,
                &execution_context,
            )
            .await
            {
                Ok(result) => {
                    println!("Non-streaming response received: {}", result);
                    assert!(!result.is_empty());
                }
                Err(e) => {
                    // Expected to fail without proper API credentials
                    println!("Expected error (no API key): {}", e);
                    assert!(
                        e.to_string().contains("API")
                            || e.to_string().contains("key")
                            || e.to_string().contains("auth")
                    );
                }
            }
        }
        Err(e) => {
            println!("Prep failed: {}", e);
            // This should succeed since we're just parsing JSON
            panic!("Prep should not fail with valid input data");
        }
    }
}

#[test]
fn test_api_config_builder() {
    // Test creating config with streaming
    let config = ApiConfig::default()
        .with_model("gpt-4".to_string())
        .with_stream(true)
        .with_max_tokens(150)
        .with_temperature(0.8);

    assert_eq!(config.model, "gpt-4");
    assert!(config.stream);
    assert_eq!(config.max_tokens, Some(150));
    assert_eq!(config.temperature, Some(0.8));

    // Test creating config without streaming
    let config2 = ApiConfig::default()
        .with_model("gpt-3.5-turbo".to_string())
        .with_stream(false);

    assert_eq!(config2.model, "gpt-3.5-turbo");
    assert!(!config2.stream);
}
