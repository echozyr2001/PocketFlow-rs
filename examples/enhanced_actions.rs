use pocketflow_rs::{Action, ActionCondition, ActionBuilder, ComparisonOperator, InMemorySharedStore};
use serde_json::json;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 PocketFlow-RS Enhanced Action System Example");
    
    // Example 1: Simple Actions (backward compatible)
    println!("\n📝 Example 1: Simple Actions");
    let simple_action = Action::simple("continue");
    let from_string: Action = "retry".into();
    let from_str: Action = "finish".into();
    
    println!("Simple actions: {}, {}, {}", simple_action, from_string, from_str);
    
    // Example 2: Parameterized Actions
    println!("\n⚙️ Example 2: Parameterized Actions");
    let mut params = HashMap::new();
    params.insert("model".to_string(), json!("gpt-4"));
    params.insert("temperature".to_string(), json!(0.7));
    params.insert("max_tokens".to_string(), json!(100));
    
    let llm_action = Action::with_params("llm_call", params);
    println!("LLM Action: {}", llm_action);
    println!("Action name: {}", llm_action.name());
    println!("Has params: {}", llm_action.has_params());
    
    if let Some(params) = llm_action.params() {
        println!("Parameters: {:?}", params);
    }
    
    // Example 3: Conditional Actions
    println!("\n🔀 Example 3: Conditional Actions");
    let condition = ActionCondition::key_exists("user_input");
    let if_true = Action::simple("process_input");
    let if_false = Action::simple("request_input");
    
    let conditional_action = Action::conditional(condition, if_true, if_false);
    println!("Conditional action: {}", conditional_action);
    
    // More complex conditions
    let numeric_condition = ActionCondition::numeric_compare(
        "temperature", 
        ComparisonOperator::GreaterThan, 
        0.5
    );
    println!("Numeric condition: {}", numeric_condition);
    
    let and_condition = ActionCondition::and(vec![
        ActionCondition::key_exists("input"),
        ActionCondition::key_equals("status", json!("ready"))
    ]);
    println!("AND condition: {}", and_condition);
    
    // Example 4: Multiple Actions
    println!("\n🔄 Example 4: Multiple Actions");
    let multi_action = Action::multiple(vec![
        Action::simple("validate_input"),
        Action::simple("process_data"),
        Action::simple("generate_response")
    ]);
    println!("Multiple actions: {}", multi_action);
    
    // Example 5: Prioritized Actions
    println!("\n⭐ Example 5: Prioritized Actions");
    let high_priority = Action::with_priority(Action::simple("critical_task"), 10);
    let low_priority = Action::with_priority(Action::simple("background_task"), 1);
    
    println!("High priority: {} (priority: {:?})", high_priority, high_priority.priority());
    println!("Low priority: {} (priority: {:?})", low_priority, low_priority.priority());
    
    // Example 6: Actions with Metadata
    println!("\n📋 Example 6: Actions with Metadata");
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), json!("user"));
    metadata.insert("timestamp".to_string(), json!("2024-01-01T00:00:00Z"));
    metadata.insert("retry_count".to_string(), json!(0));
    
    let action_with_metadata = Action::with_metadata(
        Action::simple("user_action"),
        metadata
    );
    
    println!("Action with metadata: {}", action_with_metadata);
    if let Some(meta) = action_with_metadata.metadata() {
        println!("Metadata: {:?}", meta);
    }
    
    // Example 7: Action Builder Pattern
    println!("\n🏗️ Example 7: Action Builder Pattern");
    let complex_action = ActionBuilder::new("complex_workflow")
        .with_param("model", json!("gpt-4"))
        .with_param("temperature", json!(0.8))
        .with_priority(5)
        .with_metadata({
            let mut meta = HashMap::new();
            meta.insert("created_by".to_string(), json!("system"));
            meta.insert("workflow_id".to_string(), json!("wf_123"));
            meta
        })
        .build();
    
    println!("Complex action: {}", complex_action);
    println!("  Name: {}", complex_action.name());
    println!("  Priority: {:?}", complex_action.priority());
    println!("  Has params: {}", complex_action.has_params());
    println!("  Has metadata: {}", complex_action.metadata().is_some());
    
    // Example 8: Action Type Checking
    println!("\n🔍 Example 8: Action Type Checking");
    let actions = vec![
        Action::simple("simple"),
        Action::with_params("parameterized", HashMap::new()),
        Action::conditional(
            ActionCondition::Always,
            Action::simple("yes"),
            Action::simple("no")
        ),
        Action::multiple(vec![Action::simple("a"), Action::simple("b")]),
    ];
    
    for (i, action) in actions.iter().enumerate() {
        println!("Action {}: {}", i + 1, action);
        println!("  Is simple: {}", action.is_simple());
        println!("  Is conditional: {}", action.is_conditional());
        println!("  Is multiple: {}", action.is_multiple());
        println!("  Has params: {}", action.has_params());
    }
    
    // Example 9: Serialization
    println!("\n💾 Example 9: Action Serialization");
    let action_to_serialize = Action::with_params("serialize_me", {
        let mut params = HashMap::new();
        params.insert("data".to_string(), json!({"key": "value"}));
        params
    });
    
    let json_string = serde_json::to_string_pretty(&action_to_serialize)?;
    println!("Serialized action:\n{}", json_string);
    
    let deserialized: Action = serde_json::from_str(&json_string)?;
    println!("Deserialized action: {}", deserialized);
    println!("Actions are equal: {}", action_to_serialize == deserialized);
    
    // Example 10: Integration with SharedStore
    println!("\n🔗 Example 10: Actions with SharedStore");
    let mut store = InMemorySharedStore::new();
    store.set("user_input".to_string(), json!("Hello World"))?;
    store.set("temperature".to_string(), json!(0.9))?;
    store.set("status".to_string(), json!("ready"))?;
    
    // Simulate condition evaluation (in a real implementation, you'd have an evaluator)
    let has_input = store.contains_key("user_input")?;
    let temp_value = store.get("temperature")?.and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status_ready = store.get("status")? == Some(json!("ready"));
    
    println!("Store state:");
    println!("  Has user input: {}", has_input);
    println!("  Temperature: {}", temp_value);
    println!("  Status ready: {}", status_ready);
    
    // Choose action based on conditions
    let next_action = if has_input && temp_value > 0.5 && status_ready {
        Action::with_params("process_with_llm", {
            let mut params = HashMap::new();
            params.insert("temperature".to_string(), json!(temp_value));
            params
        })
    } else {
        Action::simple("wait_for_conditions")
    };
    
    println!("Next action: {}", next_action);
    
    println!("\n✅ Enhanced Action system demonstration complete!");
    
    Ok(())
}