//! 🌌 PocketFlow-rs Hello World
//!
//! Your first PocketFlow-rs workflow demonstrating the basic execution model.
//! This example shows the fundamental concepts of nodes, shared stores, and flow execution.

use pocketflow_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌌 Welcome to PocketFlow-rs!");
    println!("Your first workflow with the three-phase execution model\n");

    // Phase 1: Create a simple workflow
    println!("📝 Building your first workflow...");

    // Create a simple greeting node wrapped in Node
    let hello_node = Node::new(LogNode::new(
        "Hello, PocketFlow-rs! 🚀",
        Action::simple("complete"),
    ));

    // Build the flow using FlowBuilder (fluent API)
    let mut flow = FlowBuilder::new()
        .start_node("start") // Entry point
        .terminal_action("complete") // Exit condition
        .node("start", hello_node) // Add our node
        .build();

    println!("✅ Flow created successfully!");

    // Phase 2: Set up shared store
    println!("\n🔧 Setting up shared store...");
    let mut store = SharedStore::new();

    // Add some initial data
    store.set(
        "user_name".to_string(),
        serde_json::json!("PocketFlow-rs Explorer"),
    )?;
    store.set("session_id".to_string(), serde_json::json!("session_001"))?;
    store.set(
        "timestamp".to_string(),
        serde_json::json!(chrono::Utc::now().to_rfc3339()),
    )?;

    println!("✅ Store initialized with session data");

    // Phase 3: Execute the workflow
    println!("\n🚀 Executing workflow...");
    let result = flow.execute(&mut store).await?;

    // Display results
    println!("\n📊 Execution Results:");
    println!("  📈 Steps executed: {}", result.steps_executed);
    println!("  🛤️  Execution path: {:?}", result.execution_path);
    println!(
        "  ⏱️  Status: {}",
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );
    println!("  🎯 Final action: {}", result.final_action);

    // Show final store contents
    println!("\n💾 Final Store Contents:");
    for key in store.keys()? {
        if let Some(value) = store.get(&key)? {
            println!("  📋 {}: {}", key, value);
        }
    }

    // Demonstrate the three-phase execution model
    println!("\n🔍 Three-Phase Execution Model Explained:");
    println!("  1️⃣  PREP Phase: Node reads data from shared store");
    println!("  2️⃣  EXEC Phase: Node performs computation/logic");
    println!("  3️⃣  POST Phase: Node writes results back to store");
    println!("\n💡 This separation ensures clean, testable, and retry-safe operations!");

    // Next steps guidance
    println!("\n🎯 What's Next?");
    println!("  📚 Try: cargo run --example 02_storage_showcase");
    println!("  📚 Then: cargo run --example 03_node_showcase");
    println!("  🌟 Build your own workflows with PocketFlow-rs!");

    Ok(())
}
