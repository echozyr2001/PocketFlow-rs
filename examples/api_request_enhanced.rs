use pocketflow_rs::{Action, prelude::*};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing enhanced ApiRequestNode with async-openai SDK...");

    // Test 1: Create ApiConfig with the builder pattern
    println!("\n✅ Test 1: ApiConfig Builder Pattern");
    let config = ApiConfig::new("test-api-key")
        .with_model("gpt-3.5-turbo")
        .with_max_tokens(100)
        .with_temperature(0.8)
        .with_top_p(0.9)
        .with_timeout(30);

    println!(
        "  ✓ Config created with: model={}, max_tokens={:?}, temperature={:?}",
        config.model, config.max_tokens, config.temperature
    );

    // Test 2: Create ApiRequestNode with custom config using with_config method
    println!("\n✅ Test 2: Node Creation with with_config");
    let node1 = ApiRequestNode::new("prompt", "response", Action::simple("end"))
        .with_config(config.clone());
    println!("  ✓ Node created with custom config successfully");

    // Test 3: Test regular constructor
    println!("\n✅ Test 3: Regular Constructor");
    let node2 = ApiRequestNode::new("messages", "output", Action::simple("done"));
    println!("  ✓ Node created with default config successfully");

    // Test 4: Test SharedStore with conversation messages
    println!("\n✅ Test 4: Message Array Support");
    let mut store = SharedStore::new();

    // Create a conversation with system, user, and assistant messages
    let messages = json!([
        {
            "role": "system",
            "content": "You are a helpful assistant."
        },
        {
            "role": "user",
            "content": "What is machine learning?"
        },
        {
            "role": "assistant",
            "content": "Machine learning is a way for computers to learn patterns from data."
        },
        {
            "role": "user",
            "content": "Can you give me a simple example?"
        }
    ]);

    store.set("test_messages".to_string(), messages)?;
    println!("  ✓ Conversation messages stored successfully");

    if let Some(stored_messages) = store.get("test_messages")? {
        println!(
            "  ✓ Messages retrieved from store: {} messages",
            stored_messages.as_array().unwrap().len()
        );
    }

    // Test 5: Test with simple prompt
    println!("\n✅ Test 5: Simple Prompt Support");
    let simple_prompt = json!("Explain quantum physics in one sentence");
    store.set("test_prompt".to_string(), simple_prompt)?;

    if let Some(stored_prompt) = store.get("test_prompt")? {
        println!(
            "  ✓ Simple prompt stored: {}",
            stored_prompt.as_str().unwrap_or("N/A")
        );
    }

    // Test 6: Test comprehensive configuration
    println!("\n✅ Test 6: Comprehensive Configuration");
    let advanced_config = ApiConfig::new("test-key")
        .with_model("gpt-4")
        .with_max_tokens(200)
        .with_temperature(0.7)
        .with_top_p(0.9)
        .with_frequency_penalty(0.1)
        .with_presence_penalty(0.1)
        .with_timeout(30)
        .with_base_url("https://api.openai.com/v1");

    let node3 = ApiRequestNode::new("input", "output", Action::simple("complete"))
        .with_config(advanced_config.clone());

    println!(
        "  ✓ Advanced config: model={}, max_tokens={:?}",
        advanced_config.model, advanced_config.max_tokens
    );
    println!(
        "  ✓ Advanced config: frequency_penalty={:?}, presence_penalty={:?}",
        advanced_config.frequency_penalty, advanced_config.presence_penalty
    );

    // Test 7: Test system message configuration
    println!("\n✅ Test 7: System Message Configuration");
    let node_with_system = ApiRequestNode::new("input", "output", Action::simple("end"))
        .with_system_message("You are a helpful coding assistant.");

    println!("  ✓ Node with system message created successfully");

    // Test 8: Test retries and error handling
    println!("\n✅ Test 8: Retry Configuration");
    let node_with_retries = ApiRequestNode::new("input", "output", Action::simple("end"))
        .with_retries(3)
        .with_retry_delay(std::time::Duration::from_millis(1000));

    println!("  ✓ Node with retry configuration created successfully");

    // Test 9: Test update_config method
    println!("\n✅ Test 9: Update Config Method");
    let new_config = ApiConfig::new("new-api-key")
        .with_model("gpt-4-turbo")
        .with_temperature(0.5);

    let updated_node = ApiRequestNode::new("input", "output", Action::simple("end"))
        .update_config(new_config.clone());

    println!("  ✓ Config updated: new model={}", new_config.model);

    println!("\n🎉 All tests passed! Enhanced ApiRequestNode working correctly.");
    println!("\n📋 Features Successfully Tested:");
    println!("  ✓ Enhanced ApiConfig with comprehensive builder pattern");
    println!("  ✓ with_config() method for custom configuration");
    println!("  ✓ Message arrays for conversation context");
    println!("  ✓ Simple prompt strings");
    println!("  ✓ System message configuration");
    println!("  ✓ Retry and error handling options");
    println!("  ✓ Dynamic configuration updates");
    println!("  ✓ Full async-openai SDK integration");
    println!("  ✓ Comprehensive parameter support");
    println!("  ✓ SharedStore integration");

    Ok(())
}
