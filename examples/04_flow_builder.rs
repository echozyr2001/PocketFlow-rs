//! 🛠️ PocketFlow-rs Builder Patterns
//!
//! Comprehensive guide to using FlowBuilder for creating production workflows.
//! Learn patterns for simple flows, conditional branching, loops, and error handling.

use pocketflow_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️ PocketFlow-rs Builder Patterns");
    println!("Mastering workflow design patterns\n");

    // 1. Simple Sequential Flow
    demo_sequential_flow().await?;
    println!("\n{}\n", "=".repeat(60));

    // 2. Conditional Branching
    demo_conditional_flow().await?;
    println!("\n{}\n", "=".repeat(60));

    // 3. Loop Pattern
    demo_loop_pattern().await?;
    println!("\n{}\n", "=".repeat(60));

    // 4. Error Handling
    demo_error_handling().await?;

    Ok(())
}

/// Pattern 1: Simple Sequential Flow
async fn demo_sequential_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Pattern 1: Sequential Flow");
    println!("   💡 Best for: Linear processes, step-by-step operations");

    let mut sequential_flow = FlowBuilder::new()
        .start_node("init") // Define entry point
        .terminal_action("complete") // Define exit condition
        .node(
            "init",
            Node::new(LogNode::new(
                "Step 1: Initialize",
                Action::simple("initialized"),
            )),
        )
        .node(
            "process",
            Node::new(LogNode::new("Step 2: Process", Action::simple("processed"))),
        )
        .node(
            "finalize",
            Node::new(LogNode::new("Step 3: Complete", Action::simple("complete"))),
        )
        .route("init", "initialized", "process") // Connect with explicit actions
        .route("process", "processed", "finalize")
        .build();

    let mut store = SharedStore::new();
    let result = sequential_flow.execute(&mut store).await?;

    println!(
        "   ✅ Executed {} steps: {:?}",
        result.steps_executed, result.execution_path
    );
    println!(
        "   📊 Result: Sequential flow → {} (Status: {})",
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Pattern 2: Conditional Branching Flow
async fn demo_conditional_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Pattern 2: Conditional Branching");
    println!("   💡 Best for: Decision trees, user personalization, A/B testing");

    let mut conditional_flow = FlowBuilder::new()
        .start_node("setup")
        .terminal_action("complete")
        .max_steps(10) // Safety limit for complex flows
        // Setup user data
        .node(
            "setup",
            Node::new(SetValueNode::new(
                "user_tier".to_string(),
                json!("premium"), // Try changing to "basic"
                Action::simple("data_ready"),
            )),
        )
        // Branch based on user tier
        .node(
            "tier_check",
            Node::new(ConditionalNode::new(
                |store| {
                    if let Ok(Some(tier)) = store.get("user_tier") {
                        let tier_str = tier.as_str().unwrap_or("basic");
                        if tier_str == "premium" {
                            println!("   🌟 Premium path selected");
                            true
                        } else {
                            println!("   👤 Standard path selected");
                            false
                        }
                    } else {
                        false
                    }
                },
                Action::simple("premium_route"),
                Action::simple("standard_route"),
            )),
        )
        // Different service responses
        .node(
            "premium_features",
            Node::new(LogNode::new(
                "🎯 Premium: Advanced analytics, priority support, custom workflows",
                Action::simple("complete"),
            )),
        )
        .node(
            "standard_features",
            Node::new(LogNode::new(
                "📋 Standard: Core features, community support, basic templates",
                Action::simple("complete"),
            )),
        )
        .route("setup", "data_ready", "tier_check")
        .route("tier_check", "premium_route", "premium_features")
        .route("tier_check", "standard_route", "standard_features")
        .build();

    let mut store = SharedStore::new();
    let result = conditional_flow.execute(&mut store).await?;

    println!(
        "   📊 Result: Conditional flow → {} (Status: {})",
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Pattern 3: Pseudo-Loop Pattern (Batch Processing)
async fn demo_loop_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Pattern 3: Batch Processing (Pseudo-Loop)");
    println!("   💡 Best for: Multi-step processing, sequential batch operations");

    let mut batch_flow = FlowBuilder::new()
        .start_node("init")
        .terminal_action("complete")
        .max_steps(20)
        // Initialize batch processing
        .node(
            "init",
            Node::new(LogNode::new(
                "📦 Starting batch processing",
                Action::simple("batch_ready"),
            )),
        )
        // Process batch 1
        .node(
            "process_batch_1",
            Node::new(SetValueNode::new(
                "batch_1_result".to_string(),
                json!({"processed": 100, "status": "complete"}),
                Action::simple("batch_1_done"),
            )),
        )
        // Process batch 2
        .node(
            "process_batch_2",
            Node::new(SetValueNode::new(
                "batch_2_result".to_string(),
                json!({"processed": 85, "status": "complete"}),
                Action::simple("batch_2_done"),
            )),
        )
        // Process batch 3
        .node(
            "process_batch_3",
            Node::new(SetValueNode::new(
                "batch_3_result".to_string(),
                json!({"processed": 92, "status": "complete"}),
                Action::simple("batch_3_done"),
            )),
        )
        // Final summary
        .node(
            "summary",
            Node::new(LogNode::new(
                "📊 All batches processed: 100 + 85 + 92 = 277 items total",
                Action::simple("complete"),
            )),
        )
        .route("init", "batch_ready", "process_batch_1")
        .route("process_batch_1", "batch_1_done", "process_batch_2")
        .route("process_batch_2", "batch_2_done", "process_batch_3")
        .route("process_batch_3", "batch_3_done", "summary")
        .build();

    let mut store = SharedStore::new();
    let result = batch_flow.execute(&mut store).await?;

    println!(
        "   📊 Result: Batch flow → {} after {} steps (Status: {})",
        result.last_node_id,
        result.steps_executed,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Pattern 4: Error Handling and Recovery
async fn demo_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ Pattern 4: Error Handling & Recovery");
    println!("   💡 Best for: Robust workflows, fault tolerance, graceful degradation");

    let mut error_flow = FlowBuilder::new()
        .start_node("risky_operation")
        .terminal_action("complete")
        .terminal_action("failed")
        .max_steps(10)
        // Simulate a risky operation that might fail
        .node(
            "risky_operation",
            Node::new(SetValueNode::new(
                "operation_result".to_string(),
                json!("success"), // Change to "error" to test error path
                Action::simple("operation_done"),
            )),
        )
        // Check operation result
        .node(
            "check_result",
            Node::new(ConditionalNode::new(
                |store| {
                    if let Ok(Some(result)) = store.get("operation_result") {
                        let result_str = result.as_str().unwrap_or("error");
                        if result_str == "success" {
                            println!("   ✅ Operation succeeded");
                            true
                        } else {
                            println!("   ⚠️ Operation failed, initiating recovery");
                            false
                        }
                    } else {
                        false
                    }
                },
                Action::simple("success_path"),
                Action::simple("error_path"),
            )),
        )
        // Success handling
        .node(
            "success_handler",
            Node::new(LogNode::new(
                "🎉 Success: Operation completed normally",
                Action::simple("complete"),
            )),
        )
        // Error recovery
        .node(
            "error_recovery",
            Node::new(LogNode::new(
                "🔧 Recovery: Fallback procedures initiated",
                Action::simple("recovered"),
            )),
        )
        // Final error handling
        .node(
            "error_notification",
            Node::new(LogNode::new(
                "📧 Error notification sent, manual intervention required",
                Action::simple("failed"),
            )),
        )
        .route("risky_operation", "operation_done", "check_result")
        .route("check_result", "success_path", "success_handler")
        .route("check_result", "error_path", "error_recovery")
        .route("error_recovery", "recovered", "error_notification")
        .build();

    let mut store = SharedStore::new();
    let result = error_flow.execute(&mut store).await?;

    println!(
        "   📊 Result: Error handling flow → {} (Status: {})",
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    // Key takeaways
    println!("\n💡 FlowBuilder Best Practices:");
    println!("  🏗️ Sequential: Use explicit action routing for clarity");
    println!("  🌳 Branching: ConditionalNode enables dynamic routing");
    println!("  🔄 Loops: Always set max_steps to prevent infinite execution");
    println!("  🛡️ Errors: Design recovery paths with multiple terminal actions");
    println!("  📊 Monitoring: Use execution_path and steps_executed for insights");

    println!("\n🎯 When to Use Each Pattern:");
    println!("  📝 Sequential → Onboarding, setup wizards, linear processes");
    println!("  🌳 Branching → User personalization, feature flags, A/B tests");
    println!("  🔄 Loops → Batch processing, retry logic, iterative workflows");
    println!("  🛡️ Error Handling → Production systems, critical operations");

    Ok(())
}
