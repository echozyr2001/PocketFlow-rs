//! 🔄 Conditional Routes & Dynamic Flow Control
//!
//! This example demonstrates the power of conditional routing in PocketFlow-rs workflows.
//! Learn how to create dynamic flows that adapt based on data and conditions.

use pocketflow_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 PocketFlow-rs Conditional Routes & Dynamic Control");
    println!("================================================\n");

    // Run multiple examples demonstrating different conditional patterns
    demo_user_auth_flow().await?;
    println!("\n{}\n", "=".repeat(60));
    demo_order_shipping_flow().await?;
    println!("\n{}\n", "=".repeat(60));
    demo_time_based_scheduling().await?;
    println!("\n{}\n", "=".repeat(60));
    demo_error_handling_flow().await?;

    Ok(())
}

/// Demo 1: User Authentication Flow
async fn demo_user_auth_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Demo 1: User Authentication Flow");
    println!("-----------------------------------");

    // Setup authentication check using conditional routing
    let setup_node = Node::new(SetValueNode::new(
        "user_authentication".to_string(),
        json!({"user_id": "user123", "role": "admin"}),
        Action::simple("check_auth"),
    ));

    let auth_check_node = Node::new(ConditionalNode::new(
        |store| {
            // Simple condition: check if authentication data exists
            if let Ok(Some(auth_data)) = store.get("user_authentication") {
                println!("   ✅ User authenticated: {}", auth_data);
                true
            } else {
                println!("   ❌ User not authenticated");
                false
            }
        },
        Action::simple("authenticated"),
        Action::simple("auth_required"),
    ));

    let dashboard_node = Node::new(LogNode::new(
        "🏠 Welcome to dashboard!",
        Action::simple("complete"),
    ));

    let login_node = Node::new(LogNode::new("🔑 Please log in", Action::simple("complete")));

    // Build authentication flow
    let mut auth_flow = FlowBuilder::new()
        .start_node("setup")
        .terminal_action("complete")
        .node("setup", setup_node)
        .node("auth_check", auth_check_node)
        .node("dashboard", dashboard_node)
        .node("login", login_node)
        .route("setup", "check_auth", "auth_check")
        .route("auth_check", "authenticated", "dashboard")
        .route("auth_check", "auth_required", "login")
        .build();

    let mut store = SharedStore::new();
    let result = auth_flow.execute(&mut store).await?;

    println!(
        "📊 Auth Flow Result: {} → {} (Status: {})",
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

/// Demo 2: Order Shipping Calculation
async fn demo_order_shipping_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Demo 2: Order Shipping Calculation");
    println!("-------------------------------------");

    let input_node = Node::new(SetValueNode::new(
        "order_total".to_string(),
        json!(125.50),
        Action::simple("calculate"),
    ));

    let shipping_calculator = Node::new(ConditionalNode::new(
        |store| {
            if let Ok(Some(total)) = store.get("order_total") {
                let total_val = total.as_f64().unwrap_or(0.0);
                println!("   📊 Order total: ${:.2}", total_val);

                if total_val >= 100.0 {
                    println!("   🆓 Qualifies for free shipping!");
                    true // Free shipping path
                } else {
                    println!("   📮 Standard shipping applies");
                    false // Standard shipping path
                }
            } else {
                false
            }
        },
        Action::simple("free_shipping"),
        Action::simple("standard_shipping"),
    ));

    let free_shipping_node = Node::new(SetValueNode::new(
        "shipping_cost".to_string(),
        json!(0.0),
        Action::simple("show_result"),
    ));

    let standard_shipping_node = Node::new(SetValueNode::new(
        "shipping_cost".to_string(),
        json!(9.99),
        Action::simple("show_result"),
    ));

    let result_node = Node::new(LogNode::new(
        "📦 Shipping calculation completed",
        Action::simple("complete"),
    ));

    let mut shipping_flow = FlowBuilder::new()
        .start_node("input")
        .terminal_action("complete")
        .node("input", input_node)
        .node("shipping_calculator", shipping_calculator)
        .node("free_shipping", free_shipping_node)
        .node("standard_shipping", standard_shipping_node)
        .node("result", result_node)
        .route("input", "calculate", "shipping_calculator")
        .route("shipping_calculator", "free_shipping", "free_shipping")
        .route(
            "shipping_calculator",
            "standard_shipping",
            "standard_shipping",
        )
        .route("free_shipping", "show_result", "result")
        .route("standard_shipping", "show_result", "result")
        .build();

    let mut store = SharedStore::new();
    let result = shipping_flow.execute(&mut store).await?;

    if let Ok(Some(shipping_cost)) = store.get("shipping_cost") {
        println!(
            "📊 Final shipping cost: ${}",
            shipping_cost.as_f64().unwrap_or(0.0)
        );
    }

    println!(
        "📊 Shipping Flow Result: {} → {} (Status: {})",
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

/// Demo 3: Time-based Task Scheduling
async fn demo_time_based_scheduling() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏰ Demo 3: Time-based Task Scheduling");
    println!("-------------------------------------");

    let scheduler_node = Node::new(ConditionalNode::new(
        |_store| {
            use chrono::{Timelike, Utc};
            let current_hour = Utc::now().hour();

            println!("   🕐 Current hour (UTC): {}", current_hour);

            match current_hour {
                6..=11 => {
                    println!("   🌅 Morning time detected");
                    true // Morning tasks
                }
                12..=17 => {
                    println!("   ☀️ Afternoon time detected");
                    false // Afternoon tasks
                }
                18..=23 => {
                    println!("   🌙 Evening time detected");
                    false // Evening tasks
                }
                _ => {
                    println!("   🌃 Night time detected");
                    false // Night tasks
                }
            }
        },
        Action::simple("morning_schedule"),
        Action::simple("other_schedule"),
    ));

    let morning_node = Node::new(LogNode::new(
        "📧 Executing morning tasks: Send digest emails",
        Action::simple("complete"),
    ));

    let other_node = Node::new(LogNode::new(
        "📊 Executing non-morning tasks: Regular operations",
        Action::simple("complete"),
    ));

    let mut schedule_flow = FlowBuilder::new()
        .start_node("scheduler")
        .terminal_action("complete")
        .node("scheduler", scheduler_node)
        .node("morning_tasks", morning_node)
        .node("other_tasks", other_node)
        .route("scheduler", "morning_schedule", "morning_tasks")
        .route("scheduler", "other_schedule", "other_tasks")
        .build();

    let mut store = SharedStore::new();
    let result = schedule_flow.execute(&mut store).await?;

    println!(
        "📊 Schedule Flow Result: {} → {} (Status: {})",
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

/// Demo 4: Error Handling & Recovery Flow
async fn demo_error_handling_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️  Demo 4: Error Handling & Recovery Flow");
    println!("-------------------------------------------");

    let operation_node = Node::new(ConditionalNode::new(
        |_store| {
            // Simulate random operation success/failure using timestamp
            use std::time::SystemTime;
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let random_val = timestamp % 100;

            if random_val < 60 {
                println!("   ✅ Operation succeeded (random: {})", random_val);
                true // Success path
            } else {
                println!("   ❌ Operation failed (random: {})", random_val);
                false // Error path
            }
        },
        Action::simple("success"),
        Action::simple("error"),
    ));

    let success_node = Node::new(LogNode::new(
        "🎉 Success! Operation completed successfully",
        Action::simple("complete"),
    ));

    let error_node = Node::new(LogNode::new(
        "💔 Error occurred, logging and alerting team",
        Action::simple("complete"),
    ));

    let mut recovery_flow = FlowBuilder::new()
        .start_node("operation")
        .terminal_action("complete")
        .node("operation", operation_node)
        .node("success_handler", success_node)
        .node("error_handler", error_node)
        .route("operation", "success", "success_handler")
        .route("operation", "error", "error_handler")
        .build();

    let mut store = SharedStore::new();
    let result = recovery_flow.execute(&mut store).await?;

    println!(
        "📊 Recovery Flow Result: {} → {} (Status: {})",
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
    println!("  🔀 Conditional routing enables dynamic workflow paths");
    println!("  📊 Store data drives flow decisions and state management");
    println!("  🎯 Each condition returns true/false to determine the next action");
    println!("  🔄 Complex business logic can be elegantly modeled as flow conditions");

    Ok(())
}
