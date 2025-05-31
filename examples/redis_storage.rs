use pocketflow_rs::prelude::*;
use serde_json::json;

// Note: This example requires a Redis server running.
// Start Redis with: docker run --rm -p 6379:6379 redis:latest
// Or install Redis locally and start it.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PocketFlow-RS Redis Storage Example");
    
    // Check if Redis is available
    let redis_url = "redis://127.0.0.1:6379/";
    println!("🔗 Connecting to Redis at: {}", redis_url);
    
    let redis_storage = match pocketflow_rs::storage::RedisStorage::new(redis_url) {
        Ok(storage) => storage,
        Err(e) => {
            eprintln!("❌ Failed to connect to Redis: {}", e);
            eprintln!("💡 Please ensure Redis is running on localhost:6379");
            eprintln!("   Docker: docker run --rm -p 6379:6379 redis:latest");
            return Ok(());
        }
    };
    
    let mut store = SharedStore::with_storage(redis_storage);
    println!("✅ Connected to Redis successfully!");

    // Example 1: Basic Redis Operations
    println!("\n📝 Example 1: Basic Redis Storage Operations");
    
    // Clear any existing data for clean demo
    store.clear()?;
    println!("🧹 Cleared existing data");
    
    // Store user session data
    let user_session = json!({
        "user_id": "user_12345",
        "username": "alice_smith", 
        "login_time": "2024-01-15T10:30:00Z",
        "preferences": {
            "language": "en",
            "theme": "dark",
            "notifications": true
        },
        "permissions": ["read", "write", "admin"]
    });
    
    store.set("user_session".to_string(), user_session)?;
    println!("✅ Stored user session data in Redis");
    
    // Retrieve and display data
    if let Some(session) = store.get("user_session")? {
        println!("👤 Retrieved user session: {}", session);
    }
    
    // Example 2: LLM Configuration Storage
    println!("\n🤖 Example 2: LLM Configuration Storage");
    
    let llm_configs = vec![
        ("gpt4_creative", json!({
            "model": "gpt-4",
            "temperature": 0.9,
            "max_tokens": 2000,
            "system_prompt": "You are a creative writing assistant."
        })),
        ("gpt4_analytical", json!({
            "model": "gpt-4",
            "temperature": 0.1,
            "max_tokens": 1500,
            "system_prompt": "You are a precise analytical assistant."
        })),
        ("claude_balanced", json!({
            "model": "claude-3",
            "temperature": 0.5,
            "max_tokens": 1800,
            "system_prompt": "You are a balanced AI assistant."
        })),
    ];
    
    // Store multiple configurations
    for (config_name, config) in llm_configs {
        store.set(format!("llm_config_{}", config_name), config)?;
    }
    println!("✅ Stored {} LLM configurations", 3);
    
    // List all stored keys
    let all_keys = store.keys()?;
    println!("🔑 All keys in Redis: {:?}", all_keys);
    
    // Example 3: Workflow State Management 
    println!("\n⚡ Example 3: Workflow State Management");
    
    // Simulate a multi-step workflow state
    let workflow_states = vec![
        ("step_1_data_validation", json!({
            "status": "completed",
            "result": "valid", 
            "timestamp": "2024-01-15T10:31:00Z",
            "validation_errors": []
        })),
        ("step_2_data_processing", json!({
            "status": "in_progress",
            "progress": 0.7,
            "timestamp": "2024-01-15T10:32:00Z",
            "processed_records": 1400
        })),
        ("step_3_report_generation", json!({
            "status": "pending",
            "timestamp": null,
            "depends_on": ["step_2_data_processing"]
        })),
    ];
    
    for (step_name, state) in workflow_states {
        store.set(format!("workflow_{}", step_name), state)?;
    }
    
    println!("✅ Stored workflow state data");
    
    // Example 4: Create and Run a Simple Flow with Redis Storage
    println!("\n🔄 Example 4: Running a Flow with Redis Storage");
    
    // Clear workflow data for clean start
    store.set("workflow_input".to_string(), json!({
        "text": "PocketFlow-rs with Redis is awesome for distributed workflows!",
        "language": "en"
    }))?;
    
    // Create nodes that work with Redis-backed storage
    let analysis_node = Node::new(FunctionNode::new(
        "TextAnalyzer".to_string(),
        // Prep: Read input from Redis store
        |store: &SharedStore<_>, _| {
            store.get("workflow_input")
                .ok()
                .flatten()
                .unwrap_or(json!({}))
        },
        // Exec: Simulate text analysis
        |input, _| {
            let text = input.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            
            Ok(json!({
                "word_count": word_count,
                "character_count": char_count,
                "language": input.get("language").unwrap_or(&json!("unknown")),
                "analysis_timestamp": "2024-01-15T10:35:00Z",
                "contains_pocketflow": text.to_lowercase().contains("pocketflow")
            }))
        },
        // Post: Store analysis results back to Redis
        |store: &mut SharedStore<_>, _, result, _| {
            store.set("analysis_result".to_string(), result)?;
            Ok(Action::simple("to_summary"))
        }
    ));
    
    let summary_node = Node::new(FunctionNode::new(
        "SummaryGenerator".to_string(),
        // Prep: Read analysis results
        |store: &SharedStore<_>, _| {
            store.get("analysis_result")
                .ok()
                .flatten()
                .unwrap_or(json!({}))
        },
        // Exec: Generate summary
        |analysis, _| {
            let word_count = analysis.get("word_count").and_then(|w| w.as_u64()).unwrap_or(0);
            let char_count = analysis.get("character_count").and_then(|c| c.as_u64()).unwrap_or(0);
            let has_pocketflow = analysis.get("contains_pocketflow").and_then(|p| p.as_bool()).unwrap_or(false);
            
            Ok(json!({
                "summary": format!("Text contains {} words and {} characters", word_count, char_count),
                "mentions_pocketflow": has_pocketflow,
                "analysis_complete": true,
                "generated_at": "2024-01-15T10:36:00Z"
            }))
        },
        // Post: Store final summary
        |store: &mut SharedStore<_>, _, result, _| {
            store.set("final_summary".to_string(), result)?;
            Ok(Action::simple("complete"))
        }
    ));
    
    // Build and run the flow
    let mut flow = FlowBuilder::new()
        .start_node("analysis")
        .node("analysis", analysis_node)
        .node("summary", summary_node)
        .route("analysis", "to_summary", "summary")
        .build();
    
    println!("🎯 Running text analysis flow with Redis storage...");
    flow.execute(&mut store).await?;
    
    // Display results
    if let Some(final_result) = store.get("final_summary")? {
        println!("📊 Final Analysis Summary: {}", final_result);
    }
    
    // Example 5: Storage Statistics and Cleanup
    println!("\n📈 Example 5: Storage Statistics");
    
    println!("📊 Redis Storage Statistics:");
    println!("  - Total keys: {}", store.len()?);
    println!("  - Is empty: {}", store.is_empty()?);
    
    // Show all keys with their prefixes
    let all_final_keys = store.keys()?;
    println!("  - All keys: {:?}", all_final_keys);
    
    // Demonstrate key removal
    println!("\n🧹 Cleaning up specific keys...");
    let removed = store.remove("workflow_input")?;
    println!("🗑️  Removed 'workflow_input': {}", removed.is_some());
    
    println!("📊 After cleanup - Total keys: {}", store.len()?);
    
    // Optional: Full cleanup (uncomment to clear all data)
    // println!("\n🧹 Clearing all data from Redis...");
    // store.clear()?;
    // println!("✅ All data cleared. Final key count: {}", store.len()?);
    
    println!("\n🎉 Redis storage example completed successfully!");
    println!("💡 Data persists in Redis - restart the example to see persistence in action!");
    
    Ok(())
}

// Helper function to test Redis connectivity
#[allow(dead_code)]
async fn test_redis_connection(url: &str) -> bool {
    match pocketflow_rs::storage::RedisStorage::new(url) {
        Ok(_) => true,
        Err(_) => false,
    }
}