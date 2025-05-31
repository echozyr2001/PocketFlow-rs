use pocketflow_rs::prelude::*;
use serde_json::Value;
use std::time::Duration;

#[tokio::main]
async fn main() -> PocketFlowResult<()> {
    println!("=== PocketFlow Node System Demo ===\n");

    // Create a shared store
    let mut store = SharedStore::new();

    // Initialize with some data
    store.set(
        "user_input".to_string(),
        Value::String("Hello, World!".to_string()),
    )?;
    store.set(
        "multiplier".to_string(),
        Value::Number(serde_json::Number::from(3)),
    )?;

    println!("Initial store state:");
    println!("user_input: {:?}", store.get("user_input")?);
    println!("multiplier: {:?}", store.get("multiplier")?);
    println!();

    // Demo 1: Log Node
    println!("=== Demo 1: Log Node ===");
    let mut log_node = Node::new(LogNode::new(
        "Processing user input...",
        Action::simple("continue"),
    ));

    let action = log_node
        .run(&mut store)
        .await
        .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
    println!("Log node returned action: {:?}\n", action);

    // Demo 2: Get Value Node (with transformation)
    println!("=== Demo 2: Get Value Node (with transformation) ===");
    let mut transform_node = Node::new(GetValueNode::new(
        "user_input".to_string(),
        "processed_input".to_string(),
        |value: Option<Value>| -> Value {
            match value {
                Some(Value::String(s)) => Value::String(format!("Processed: {}", s)),
                _ => Value::String("Processed: <unknown>".to_string()),
            }
        },
        Action::simple("transform_complete"),
    ));

    let action = transform_node
        .run(&mut store)
        .await
        .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
    println!("Transform node returned action: {:?}", action);
    println!("Processed result: {:?}\n", store.get("processed_input")?);

    // Demo 3: Conditional Node
    println!("=== Demo 3: Conditional Node ===");
    let mut conditional_node = Node::new(ConditionalNode::new(
        |store: &SharedStore<_>| -> bool {
            if let Ok(Some(Value::Number(n))) = store.get("multiplier") {
                n.as_u64().unwrap_or(0) > 2
            } else {
                false
            }
        },
        Action::simple("high_multiplier"),
        Action::simple("low_multiplier"),
    ));

    let action = conditional_node
        .run(&mut store)
        .await
        .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
    println!("Conditional node returned action: {:?}\n", action);

    // Demo 4: Mock LLM Node with retries
    println!("=== Demo 4: Mock LLM Node with Retries ===");
    #[cfg(feature = "builtin-llm")]
    {
        let mut llm_node = Node::new(
            MockLlmNode::new(
                "processed_input".to_string(),
                "llm_response".to_string(),
                "AI Response".to_string(),
                Action::simple("llm_complete"),
            )
            .with_failure_rate(0.7) // 70% failure rate to test retries
            .with_retries(3),
        );

        let action = llm_node
            .run(&mut store)
            .await
            .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
        println!("LLM node returned action: {:?}", action);
        println!("LLM response: {:?}\n", store.get("llm_response")?);
    }

    #[cfg(not(feature = "builtin-llm"))]
    {
        println!("MockLlmNode is not available without 'builtin-llm' feature");
        store.set(
            "llm_response".to_string(),
            Value::String("Mock AI Response (no LLM feature)".to_string()),
        )?;
        println!("Set mock response instead\n");
    };

    // Demo 5: Delay Node
    println!("=== Demo 5: Delay Node ===");
    let start = std::time::Instant::now();
    let mut delay_node = Node::new(DelayNode::new(
        Duration::from_millis(500),
        Action::simple("delay_complete"),
    ));

    let action = delay_node
        .run(&mut store)
        .await
        .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
    let elapsed = start.elapsed();
    println!("Delay node returned action: {:?}", action);
    println!("Actual delay: {:?}\n", elapsed);

    // Demo 6: Set Value Node
    println!("=== Demo 6: Set Value Node ===");
    let mut set_node = Node::new(SetValueNode::new(
        "final_result".to_string(),
        Value::String("Workflow completed successfully!".to_string()),
        Action::simple("workflow_complete"),
    ));

    let action = set_node
        .run(&mut store)
        .await
        .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
    println!("Set node returned action: {:?}", action);
    println!("Final result: {:?}\n", store.get("final_result")?);

    // Demo 7: Custom Function Node
    println!("=== Demo 7: Custom Function Node ===");
    let mut function_node = Node::new(
        FunctionNode::new(
            "CustomMathNode".to_string(),
            // Prep: read multiplier from store
            |store: &SharedStore<_>, _context: &ExecutionContext| -> i32 {
                store
                    .get("multiplier")
                    .ok()
                    .flatten()
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32
            },
            // Exec: perform calculation
            |multiplier: i32,
             _context: &ExecutionContext|
             -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
                Ok(multiplier * 42)
            },
            // Post: store result
            |store: &mut SharedStore<_>,
             _prep: i32,
             result: i32,
             _context: &ExecutionContext|
             -> Result<Action, Box<dyn std::error::Error + Send + Sync>> {
                store.set(
                    "calculation_result".to_string(),
                    Value::Number(serde_json::Number::from(result)),
                )?;
                Ok(Action::simple("calculation_complete"))
            },
        )
        .with_retries(2),
    );

    let action = function_node
        .run(&mut store)
        .await
        .map_err(|e| PocketFlowError::ExecutionError(e.to_string()))?;
    println!("Function node returned action: {:?}", action);
    println!(
        "Calculation result: {:?}\n",
        store.get("calculation_result")?
    );

    // Final store state
    println!("=== Final Store State ===");
    for key in [
        "user_input",
        "processed_input",
        "llm_response",
        "final_result",
        "calculation_result",
    ] {
        if let Ok(Some(value)) = store.get(key) {
            println!("{}: {:?}", key, value);
        }
    }

    println!("\n=== Node System Demo Complete ===");
    Ok(())
}
