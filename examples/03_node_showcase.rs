//! 🧩 PocketFlow-rs Node Showcase
//!
//! Comprehensive demonstration of built-in node types and their capabilities.
//! Learn how to use different node types effectively in your workflows.

use pocketflow_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧩 PocketFlow-rs Node Showcase");
    println!("Exploring different node types and capabilities\n");

    // Demo 1: Basic Built-in Nodes
    demo_basic_nodes().await?;
    println!("\n{}\n", "=".repeat(60));

    // Demo 2: Conditional Logic
    demo_conditional_nodes().await?;
    println!("\n{}\n", "=".repeat(60));

    // Demo 3: Data Management
    demo_data_nodes().await?;

    Ok(())
}

/// Demo 1: Basic Built-in Node Types
async fn demo_basic_nodes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️ Demo 1: Basic Built-in Node Types");
    println!("------------------------------------");

    // LogNode: Simple logging and messaging
    let start_node = Node::new(LogNode::new(
        "🚀 Starting basic nodes demonstration",
        Action::simple("logged"),
    ));

    // DelayNode: Timing and throttling
    let delay_node = Node::new(DelayNode::new(
        std::time::Duration::from_millis(100),
        Action::simple("delayed"),
    ));

    // Final log
    let end_node = Node::new(LogNode::new(
        "✅ Basic nodes demonstration completed",
        Action::simple("complete"),
    ));

    // Build and execute flow
    let mut flow = FlowBuilder::new()
        .start_node("start")
        .terminal_action("complete")
        .node("start", start_node)
        .node("delay", delay_node)
        .node("end", end_node)
        .route("start", "logged", "delay")
        .route("delay", "delayed", "end")
        .build();

    let mut store = SharedStore::new();
    let result = flow.execute(&mut store).await?;

    println!(
        "📊 Basic Nodes Result: {} → {} (Status: {})",
        result.execution_path.join(" → "),
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Demo 2: Conditional Logic and Decision Making
async fn demo_conditional_nodes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Demo 2: Conditional Logic & Decision Making");
    println!("-----------------------------------------------");

    // Setup some test data
    let setup_node = Node::new(SetValueNode::new(
        "user_score".to_string(),
        json!(85),
        Action::simple("data_ready"),
    ));

    // Conditional routing based on score
    let score_check_node = Node::new(ConditionalNode::new(
        |store| {
            if let Ok(Some(score_val)) = store.get("user_score") {
                let score = score_val.as_i64().unwrap_or(0);
                println!("   📊 User score: {}", score);

                if score >= 80 {
                    println!("   🌟 High score detected!");
                    true // High score path
                } else {
                    println!("   📈 Standard score");
                    false // Standard score path
                }
            } else {
                false
            }
        },
        Action::simple("high_score"),
        Action::simple("standard_score"),
    ));

    // Different responses for different scores
    let high_score_node = Node::new(LogNode::new(
        "🎉 Excellent performance! You qualify for premium features",
        Action::simple("complete"),
    ));

    let standard_score_node = Node::new(LogNode::new(
        "💪 Good work! Keep improving to unlock more features",
        Action::simple("complete"),
    ));

    let mut flow = FlowBuilder::new()
        .start_node("setup")
        .terminal_action("complete")
        .node("setup", setup_node)
        .node("score_check", score_check_node)
        .node("high_score", high_score_node)
        .node("standard_score", standard_score_node)
        .route("setup", "data_ready", "score_check")
        .route("score_check", "high_score", "high_score")
        .route("score_check", "standard_score", "standard_score")
        .build();

    let mut store = SharedStore::new();
    let result = flow.execute(&mut store).await?;

    println!(
        "📊 Conditional Result: {} → {} (Status: {})",
        result.execution_path.join(" → "),
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Demo 3: Data Management with SetValueNode
async fn demo_data_nodes() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Demo 3: Data Management & Storage");
    println!("------------------------------------");

    // Create comprehensive user profile
    let profile_setup_node = Node::new(SetValueNode::new(
        "user_profile".to_string(),
        json!({
            "id": "user_001",
            "name": "Alice Johnson",
            "email": "alice@example.com",
            "role": "developer",
            "permissions": ["read", "write", "admin"],
            "preferences": {
                "theme": "dark",
                "notifications": true,
                "timezone": "UTC"
            },
            "stats": {
                "login_count": 42,
                "last_login": "2024-01-15T10:30:00Z",
                "projects": 7
            }
        }),
        Action::simple("profile_created"),
    ));

    // Add session data
    let session_node = Node::new(SetValueNode::new(
        "session_info".to_string(),
        json!({
            "session_id": "sess_123456",
            "start_time": chrono::Utc::now().to_rfc3339(),
            "ip_address": "192.168.1.100",
            "device": "desktop",
            "browser": "Chrome/121.0"
        }),
        Action::simple("session_created"),
    ));

    // Process and validate the data
    let validation_node = Node::new(ConditionalNode::new(
        |store| {
            // Check if we have both profile and session data
            let has_profile = store.get("user_profile").unwrap_or(None).is_some();
            let has_session = store.get("session_info").unwrap_or(None).is_some();

            if has_profile && has_session {
                println!("   ✅ All data validation passed");
                true
            } else {
                println!("   ❌ Data validation failed");
                false
            }
        },
        Action::simple("validation_success"),
        Action::simple("validation_failed"),
    ));

    // Success summary
    let success_node = Node::new(LogNode::new(
        "📋 Complete user context established - ready for application flow",
        Action::simple("complete"),
    ));

    // Error handling
    let error_node = Node::new(LogNode::new(
        "❌ Data setup failed - check configuration",
        Action::simple("complete"),
    ));

    let mut flow = FlowBuilder::new()
        .start_node("profile_setup")
        .terminal_action("complete")
        .node("profile_setup", profile_setup_node)
        .node("session_setup", session_node)
        .node("validation", validation_node)
        .node("success", success_node)
        .node("error", error_node)
        .route("profile_setup", "profile_created", "session_setup")
        .route("session_setup", "session_created", "validation")
        .route("validation", "validation_success", "success")
        .route("validation", "validation_failed", "error")
        .build();

    let mut store = SharedStore::new();
    let result = flow.execute(&mut store).await?;

    // Show final stored data summary
    println!("\n💾 Final Store Contents:");
    if let Ok(Some(profile)) = store.get("user_profile") {
        println!(
            "  👤 User: {} ({})",
            profile["name"].as_str().unwrap_or("Unknown"),
            profile["role"].as_str().unwrap_or("No Role")
        );
    }

    if let Ok(Some(session)) = store.get("session_info") {
        println!(
            "  🔐 Session: {} from {}",
            session["session_id"].as_str().unwrap_or("Unknown"),
            session["device"].as_str().unwrap_or("Unknown")
        );
    }

    println!(
        "📊 Data Management Result: {} → {} (Status: {})",
        result.execution_path.join(" → "),
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    // Key takeaways
    println!("\n💡 Key Takeaways:");
    println!("  🏗️ LogNode: Perfect for debugging, user feedback, and process status");
    println!("  ⏱️  DelayNode: Essential for rate limiting and timing control");
    println!("  💾 SetValueNode: Core tool for state management and data flow");
    println!("  🔀 ConditionalNode: Enables smart workflows that adapt to data");
    println!("  📊 Three-phase execution: Prep → Exec → Post ensures reliability");

    println!("\n🎯 Node Selection Guide:");
    println!("  📝 Need to show messages? → LogNode");
    println!("  💾 Need to store data? → SetValueNode");
    println!("  🔀 Need conditional logic? → ConditionalNode");
    println!("  ⏱️  Need timing control? → DelayNode");
    println!("  🎨 Need custom logic? → Implement NodeBackend trait");

    Ok(())
}
