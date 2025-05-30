use pocketflow_rs::{
    ApiRequestNode, ApiConfig, Node, FlowBuilder, Action, Flow, NodeBackend,
    SharedStore, SetValueNode, LogNode,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 PocketFlow API Request Node Example");
    println!("=====================================\n");
    
    // Example 1: Basic API Request Node Usage
    println!("📝 Example 1: Basic API Request Node");
    println!("-----------------------------------");
    
    let config = ApiConfig {
        api_key: std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "demo-key-for-testing".to_string()),
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        ..Default::default()
    };
    
    let mut api_node = Node::new(
        ApiRequestNode::new(
            config.clone(),
            "user_question",
            "ai_answer",
            Action::simple("complete")
        )
        .with_system_message("You are a helpful assistant. Provide concise answers.")
        .with_retries(2)
    );
    
    let mut store = SharedStore::new();
    store.set("user_question".to_string(), json!("What is the capital of France?"))?;
    
    println!("🔍 Question: What is the capital of France?");
    
    match api_node.run(&mut store).await {
        Ok(action) => {
            let answer = store.get("ai_answer")?;
            println!("🤖 AI Answer: {:?}", answer);
            println!("⚡ Action: {:?}", action);
            
            // Check for usage information
            if let Ok(Some(usage)) = store.get("ai_answer_usage") {
                println!("📊 Usage: {:?}", usage);
            }
            
            if let Ok(Some(model)) = store.get("ai_answer_model") {
                println!("🔧 Model: {:?}", model);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            // Even on error, check if fallback response was stored
            if let Ok(Some(fallback)) = store.get("ai_answer") {
                println!("🔄 Fallback response: {:?}", fallback);
            }
        }
    }
    
    println!("\n🏭 Example 2: AI-Powered Flow");
    println!("-----------------------------");
    
    // Create a flow that processes user input through AI
    let input_node = Node::new(SetValueNode::new(
        "user_input".to_string(),
        json!("Explain quantum computing in simple terms"),
        Action::simple("to_ai")
    ));
    
    let ai_config = ApiConfig {
        api_key: std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "demo-key-for-testing".to_string()),
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(200),
        temperature: Some(0.5),
        ..Default::default()
    };
    
    let ai_node = Node::new(
        ApiRequestNode::new(
            ai_config,
            "user_input",
            "ai_explanation",
            Action::simple("to_summary")
        )
        .with_system_message("You are an expert teacher. Explain complex topics in simple, easy-to-understand language.")
    );
    
    let summary_node = Node::new(LogNode::new(
        "AI processing complete!",
        Action::simple("done")
    ));
    
    let mut flow = FlowBuilder::new()
        .start_node("input")
        .node("input", input_node)
        .node("ai_process", ai_node)
        .node("summary", summary_node)
        .route("input", "to_ai", "ai_process")
        .route("ai_process", "to_summary", "summary")
        .build();
    
    let mut flow_store = SharedStore::new();
    
    println!("🔍 Processing: Explain quantum computing in simple terms");
    
    match flow.execute(&mut flow_store).await {
        Ok(result) => {
            println!("✅ Flow completed successfully!");
            println!("📊 Steps executed: {}", result.steps_executed);
            println!("🛤️  Execution path: {:?}", result.execution_path);
            
            if let Ok(Some(explanation)) = flow_store.get("ai_explanation") {
                println!("🤖 AI Explanation: {:?}", explanation);
            }
        }
        Err(e) => {
            println!("❌ Flow error: {}", e);
            // Check if we got a partial result
            if let Ok(Some(partial)) = flow_store.get("ai_explanation") {
                println!("🔄 Partial result: {:?}", partial);
            }
        }
    }
    
    println!("\n🔧 Example 3: Custom API Configuration");
    println!("-------------------------------------");
    
    // Example with custom base URL (for local or alternative APIs)
    let custom_config = ApiConfig {
        api_key: "custom-key".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()), // Default OpenAI URL
        model: "gpt-4".to_string(),
        max_tokens: Some(150),
        temperature: Some(0.3),
        timeout: Some(60),
        ..Default::default()
    };
    
    println!("🔧 Custom configuration:");
    println!("   Model: {}", custom_config.model);
    println!("   Max tokens: {:?}", custom_config.max_tokens);
    println!("   Temperature: {:?}", custom_config.temperature);
    println!("   Base URL: {:?}", custom_config.base_url);
    
    let custom_node = ApiRequestNode::new(
        custom_config,
        "prompt",
        "response",
        Action::simple("done")
    )
    .with_system_message("You are a creative writing assistant.")
    .with_retries(3);
    
    println!("✅ Custom API node created with 3 max retries");
    
    println!("\n💡 Tips for using ApiRequestNode:");
    println!("   • Set OPENAI_API_KEY environment variable for real API calls");
    println!("   • Use system messages to guide AI behavior");
    println!("   • Configure retries for robust error handling");
    println!("   • Store API usage information for monitoring");
    println!("   • Use custom base URLs for alternative API providers");
    
    Ok(())
}