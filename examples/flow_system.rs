use pocketflow_rs::prelude::*;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PocketFlow-RS Flow System Example ===\n");

    // Example 1: Simple Linear Flow
    println!("1. Simple Linear Flow:");
    simple_linear_flow().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // Example 2: Conditional Flow
    println!("2. Conditional Flow:");
    conditional_flow().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // Example 3: Complex Workflow
    println!("3. Complex Workflow:");
    complex_workflow().await?;
    
    Ok(())
}

async fn simple_linear_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Create nodes
    let start_node = Node::new(LogNode::new("Starting workflow...", Action::simple("init")));
    let process_node = Node::new(SetValueNode::new(
        "status".to_string(),
        json!("processing"),
        Action::simple("validate")
    ));
    let end_node = Node::new(LogNode::new("Workflow completed!", Action::simple("complete")));
    
    // Build flow using the builder pattern
    let mut flow = FlowBuilder::new()
        .start_node("start")
        .terminal_action("complete")
        .node("start", start_node)
        .node("process", process_node)
        .node("end", end_node)
        .route("start", "init", "process")
        .route("process", "validate", "end")
        .build();
    
    // Execute the flow
    let mut store = SharedStore::new();
    let result = flow.execute(&mut store).await?;
    
    println!("  Execution completed in {} steps", result.steps_executed);
    println!("  Execution path: {:?}", result.execution_path);
    println!("  Final status: {:?}", store.get("status")?);
    
    Ok(())
}

async fn conditional_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Create a setup node that sets a condition
    let setup_node = Node::new(SetValueNode::new(
        "user_authenticated".to_string(),
        json!(true), // Try changing this to false
        Action::simple("check_auth")
    ));
    
    // Create success and failure paths
    let success_node = Node::new(SetValueNode::new(
        "result".to_string(),
        json!("Access granted"),
        Action::simple("complete")
    ));
    
    let failure_node = Node::new(SetValueNode::new(
        "result".to_string(),
        json!("Access denied"),
        Action::simple("complete")
    ));
    
    // Build conditional flow
    let mut flow = FlowBuilder::new()
        .start_node("setup")
        .node("setup", setup_node)
        .node("success", success_node)
        .node("failure", failure_node)
        .conditional_route(
            "setup", 
            "check_auth", 
            "success", 
            RouteCondition::KeyEquals("user_authenticated".to_string(), json!(true))
        )
        .conditional_route(
            "setup", 
            "check_auth", 
            "failure", 
            RouteCondition::KeyEquals("user_authenticated".to_string(), json!(false))
        )
        .build();
    
    let mut store = SharedStore::new();
    let result = flow.execute(&mut store).await?;
    
    println!("  Authentication result: {:?}", store.get("result")?);
    println!("  Execution path: {:?}", result.execution_path);
    
    Ok(())
}

async fn complex_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Create a complex workflow with multiple paths and retries
    
    // Input validation node
    let input_node = Node::new(SetValueNode::new(
        "input_data".to_string(),
        json!({"user_id": 123, "action": "process"}),
        Action::simple("validate")
    ));
    
    // Validation node with conditional routing
    let validation_node = Node::new(FunctionNode::new(
        "ValidationNode".to_string(),
        // Prep: read input data
        |store: &SharedStore<_>, _context: &ExecutionContext| -> serde_json::Value {
            store.get("input_data")
                .ok()
                .flatten()
                .unwrap_or(json!({}))
        },
        // Exec: validate the data
        |data: serde_json::Value, _context: &ExecutionContext| -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            let is_valid = data.get("user_id").and_then(|v| v.as_i64()).is_some() &&
                          data.get("action").and_then(|v| v.as_str()).is_some();
            Ok(is_valid)
        },
        // Post: set validation result and route
        |store: &mut SharedStore<_>, _prep: serde_json::Value, is_valid: bool, _context: &ExecutionContext| -> Result<Action, Box<dyn std::error::Error + Send + Sync>> {
            store.set("validation_passed".to_string(), json!(is_valid))?;
            if is_valid {
                // Set a prompt for the LLM node
                store.set("prompt".to_string(), json!("Process this data: {}"))?;
                Ok(Action::simple("process"))
            } else {
                Ok(Action::simple("error"))
            }
        }
    ));
    
    // Processing node with simulated LLM call
    let processing_node = Node::new(MockLlmNode::new(
        "prompt".to_string(), // Use a different key for the prompt
        "llm_response".to_string(),
        "AI processed the data successfully".to_string(),
        Action::simple("finalize")
    ).with_failure_rate(0.2).with_retries(3));
    
    // Error handling node
    let error_node = Node::new(SetValueNode::new(
        "error_message".to_string(),
        json!("Validation failed"),
        Action::simple("complete")
    ));
    
    // Final result node
    let finalize_node = Node::new(FunctionNode::new(
        "FinalizeNode".to_string(),
        // Prep: gather all results
        |store: &SharedStore<_>, _context: &ExecutionContext| -> (Option<serde_json::Value>, Option<serde_json::Value>) {
            let input = store.get("input_data").ok().flatten();
            let response = store.get("llm_response").ok().flatten();
            (input, response)
        },
        // Exec: create final result
        |data: (Option<serde_json::Value>, Option<serde_json::Value>), _context: &ExecutionContext| -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
            let result = json!({
                "status": "success",
                "input": data.0,
                "ai_response": data.1,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            Ok(result)
        },
        // Post: store final result
        |store: &mut SharedStore<_>, _prep: (Option<serde_json::Value>, Option<serde_json::Value>), result: serde_json::Value, _context: &ExecutionContext| -> Result<Action, Box<dyn std::error::Error + Send + Sync>> {
            store.set("final_result".to_string(), result)?;
            Ok(Action::simple("complete"))
        }
    ));
    
    // Add a delay node for demonstration
    let delay_node = Node::new(DelayNode::new(
        Duration::from_millis(100),
        Action::simple("continue")
    ));
    
    // Build the complex flow
    let mut flow = FlowBuilder::new()
        .start_node("input")
        .max_steps(20)
        .node("input", input_node)
        .node("validate", validation_node)
        .node("delay", delay_node)
        .node("process", processing_node)
        .node("error", error_node)
        .node("finalize", finalize_node)
        .route("input", "validate", "validate")
        .conditional_route(
            "validate", 
            "process", 
            "delay", 
            RouteCondition::KeyEquals("validation_passed".to_string(), json!(true))
        )
        .conditional_route(
            "validate", 
            "error", 
            "error", 
            RouteCondition::KeyEquals("validation_passed".to_string(), json!(false))
        )
        .route("delay", "continue", "process")
        .route("process", "finalize", "finalize")
        .build();
    
    // Validate the flow before execution
    flow.validate()?;
    
    let mut store = SharedStore::new();
    let result = flow.execute(&mut store).await?;
    
    println!("  Complex workflow completed!");
    println!("  Steps executed: {}", result.steps_executed);
    println!("  Execution path: {:?}", result.execution_path);
    
    if let Some(final_result) = store.get("final_result")? {
        println!("  Final result: {}", serde_json::to_string_pretty(&final_result)?);
    }
    
    if let Some(error_msg) = store.get("error_message")? {
        println!("  Error: {:?}", error_msg);
    }
    
    Ok(())
}