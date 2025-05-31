use pocketflow_rs::{Action, Node, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PocketFlow Enhanced API Request Example");
    println!("==========================================");

    // Create storage
    let mut store = SharedStore::new();

    // Demo 1: Simple prompt usage
    println!("\n📝 Demo 1: Simple Prompt");
    println!("========================");

    store.set(
        "simple_prompt".to_string(),
        serde_json::json!("What is the capital of France?"),
    )?;

    // Demo 2: Message array with context
    println!("\n💬 Demo 2: Message Array with Context");
    println!("=====================================");

    let conversation_messages = serde_json::json!([
        {
            "role": "system",
            "content": "You are a helpful geography expert."
        },
        {
            "role": "user",
            "content": "What is the capital of France?"
        },
        {
            "role": "assistant",
            "content": "The capital of France is Paris."
        },
        {
            "role": "user",
            "content": "What is its population?"
        }
    ]);

    store.set("conversation".to_string(), conversation_messages)?;

    // Check for API key
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo_key".to_string());

    if api_key == "demo_key" {
        println!("ℹ️  No OPENAI_API_KEY environment variable found.");
        println!("   Using MockLlmNode for demonstration purposes.");
        println!("   To use actual OpenAI API, set OPENAI_API_KEY=your_key\n");

        // Demo with MockLlmNode for simple prompt
        let mock_node = Node::new(MockLlmNode::new(
            "simple_prompt",
            "simple_response",
            "This is a mock response: Paris is the capital of France.",
            Action::simple("end"),
        ));

        let mut flow = FlowBuilder::new()
            .start_node("mock_simple")
            .node("mock_simple", mock_node)
            .build();

        println!("🔄 Executing simple prompt with MockLlmNode...");
        let result = flow.execute(&mut store).await?;
        println!("✅ Flow completed with result: {:?}", result);

        if let Some(response) = store.get("simple_response")? {
            println!("🤖 Mock LLM Response: {}", response);
        }
    } else {
        println!("🔑 Using OpenAI API with provided key\n");

        // Demo 1: Create API node with default config for simple prompt
        let simple_api_node = Node::new(
            ApiRequestNode::new(
                "simple_prompt",
                "simple_response",
                Action::simple("conversation_demo"),
            )
            .with_retries(2)
            .with_retry_delay(std::time::Duration::from_millis(500)),
        );

        // Demo 2: Create API node with custom config for conversation
        let api_config = ApiConfig::new(&api_key)
            .with_model("gpt-3.5-turbo")
            .with_max_tokens(200)
            .with_temperature(0.8)
            .with_timeout(30);

        let conversation_api_node = Node::new(
            ApiRequestNode::new(
                "conversation",
                "conversation_response",
                Action::simple("system_message_demo"),
            )
            .with_config(api_config)
            .with_retries(3)
            .with_system_message("You are a helpful AI assistant. Be concise in your responses."),
        );

        // Demo 3: Using system message in constructor
        store.set(
            "question".to_string(),
            serde_json::json!("Explain quantum physics in one sentence."),
        )?;

        let system_api_node = Node::new(
            ApiRequestNode::new(
                "question",
                "physics_response",
                Action::simple("update_config_demo"),
            )
            .with_system_message(
                "You are a physics professor. Explain concepts clearly but concisely.",
            )
            .with_retries(1),
        );

        // Demo 4: Updating configuration dynamically
        let new_config = ApiConfig::new(&api_key)
            .with_model("gpt-3.5-turbo")
            .with_temperature(0.3) // Lower temperature for more focused responses
            .with_max_tokens(100);

        store.set(
            "final_question".to_string(),
            serde_json::json!("What is 2+2?"),
        )?;

        let updated_config_node = Node::new(
            ApiRequestNode::new("final_question", "math_response", Action::simple("end"))
                .update_config(new_config)
                .with_retries(1),
        );

        // Create comprehensive flow
        let mut flow = FlowBuilder::new()
            .start_node("simple_prompt")
            .node("simple_prompt", simple_api_node)
            .node("conversation_demo", conversation_api_node)
            .node("system_message_demo", system_api_node)
            .node("update_config_demo", updated_config_node)
            .route("simple_prompt", "conversation_demo", "conversation_demo")
            .route(
                "conversation_demo",
                "system_message_demo",
                "system_message_demo",
            )
            .route(
                "system_message_demo",
                "update_config_demo",
                "update_config_demo",
            )
            .build();

        println!("🔄 Executing comprehensive API demo flow...");
        match flow.execute(&mut store).await {
            Ok(result) => {
                println!("✅ Flow completed with result: {:?}\n", result);

                println!("📊 Results Summary:");
                println!("==================");

                if let Some(simple_resp) = store.get("simple_response")? {
                    println!(
                        "🟢 Simple Prompt Response: {}",
                        simple_resp.as_str().unwrap_or("N/A")
                    );
                }

                if let Some(conv_resp) = store.get("conversation_response")? {
                    println!(
                        "🟢 Conversation Response: {}",
                        conv_resp.as_str().unwrap_or("N/A")
                    );
                }

                if let Some(physics_resp) = store.get("physics_response")? {
                    println!(
                        "🟢 Physics Response: {}",
                        physics_resp.as_str().unwrap_or("N/A")
                    );
                }

                if let Some(math_resp) = store.get("math_response")? {
                    println!("🟢 Math Response: {}", math_resp.as_str().unwrap_or("N/A"));
                }
            }
            Err(e) => {
                println!("❌ Flow execution failed: {}", e);
                println!("   This might be due to:");
                println!("   - Invalid API key");
                println!("   - Network issues");
                println!("   - Rate limiting");
                println!("   - API service unavailable");

                // Show any fallback responses that might have been generated
                for key in [
                    "simple_response",
                    "conversation_response",
                    "physics_response",
                    "math_response",
                ] {
                    if let Ok(Some(response)) = store.get(key) {
                        println!("🔄 Fallback for {}: {}", key, response);
                    }
                }
            }
        }
    }

    println!("\n📚 Enhanced Features Demonstrated:");
    println!("==================================");
    println!("   ✨ Simple prompt input (string)");
    println!("   ✨ Message array input with conversation history");
    println!("   ✨ System message configuration");
    println!("   ✨ with_config() method for custom API settings");
    println!("   ✨ update_config() method for dynamic configuration");
    println!("   ✨ async-openai SDK integration");
    println!("   ✨ Enhanced error handling and retries");
    println!("   ✨ Multiple parameter configurations (temperature, max_tokens, etc.)");
    println!("   ✨ Environment-based API key loading");
    println!("   ✨ Graceful fallbacks and comprehensive logging");

    Ok(())
}
