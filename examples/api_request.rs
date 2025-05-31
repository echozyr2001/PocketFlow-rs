use pocketflow_rs::{prelude::*, Action, Node};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PocketFlow API Request Example");
    println!("=================================");

    // Create storage
    let mut store = SharedStore::new();

    // Set up initial prompt
    store.set("prompt".to_string(), serde_json::json!("What is the capital of France?"))?;

    // Configure API settings (you need to provide a valid API key)
    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "demo_key".to_string());

    if api_key == "demo_key" {
        println!("ℹ️  No OPENAI_API_KEY environment variable found.");
        println!("   Using MockLlmNode for demonstration purposes.");
        println!("   To use actual OpenAI API, set OPENAI_API_KEY=your_key\n");
        
        // Use MockLlmNode as fallback for demo
        let mock_node = Node::new(MockLlmNode::new(
            "prompt",
            "response",
            "This is a mock response: Paris is the capital of France.",
            Action::simple("end"),
        ));

        // Create flow with mock node
        let mut flow = FlowBuilder::new()
            .start_node("mock_api")
            .node("mock_api", mock_node)
            .build();

        println!("🔄 Executing flow with MockLlmNode...");
        let result = flow.execute(&mut store).await?;
        println!("✅ Flow completed with result: {:?}", result);
        
        if let Some(response) = store.get("response")? {
            println!("🤖 Mock LLM Response: {}", response);
        }
    } else {
        println!("🔑 Using OpenAI API with provided key");
        
        // Create API configuration
        let api_config = ApiConfig {
            api_key,
            base_url: None, // Use default OpenAI endpoint
            org_id: None,
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: Some(150),
            temperature: Some(0.7),
            timeout: Some(30),
        };

        // Create API request node
        let api_node = Node::new(ApiRequestNode::new(
            api_config,
            "prompt", 
            "response",
            Action::simple("process_response")
        ).with_retries(2));

        // Create a processing node to handle the response
        let process_node = Node::new(LogNode::new(
            "Processing API response",
            Action::simple("end")
        ));

        // Create flow
        let mut flow = FlowBuilder::new()
            .start_node("api_request")
            .node("api_request", api_node)
            .node("process_response", process_node)
            .route("api_request", "process_response", "process_response")
            .build();

        println!("🔄 Executing flow with real API request...");
        match flow.execute(&mut store).await {
            Ok(result) => {
                println!("✅ Flow completed with result: {:?}", result);
                
                if let Some(response) = store.get("response")? {
                    println!("🤖 OpenAI Response: {}", response);
                }
            }
            Err(e) => {
                println!("❌ Flow execution failed: {}", e);
                println!("   This might be due to:");
                println!("   - Invalid API key");
                println!("   - Network issues");
                println!("   - Rate limiting");
                println!("   - API service unavailable");
                
                // Show any fallback response that might have been generated
                if let Ok(Some(response)) = store.get("response") {
                    println!("🔄 Fallback response: {}", response);
                }
            }
        }
    }

    println!("\n📚 Example Features Demonstrated:");
    println!("   ✨ ApiRequestNode for real LLM API calls");
    println!("   ✨ Automatic retries on failure");
    println!("   ✨ Graceful error handling and fallbacks");
    println!("   ✨ Environment-based configuration");
    println!("   ✨ MockLlmNode for development/testing");

    Ok(())
}