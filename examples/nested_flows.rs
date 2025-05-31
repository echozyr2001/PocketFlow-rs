use pocketflow_rs::{Action, Flow, FlowBuilder, FlowNode, Node, SetValueNode, SharedStore};
use serde_json::json;

/// This example demonstrates how to create nested flows in PocketFlow.
/// Nested flows allow you to compose complex workflows by treating entire flows as nodes.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 PocketFlow Nested Flows Example");
    println!("=====================================");

    // Example 1: Basic nested flow
    basic_nested_flow_example().await?;

    // Example 2: Data processing pipeline with nested flows
    data_processing_pipeline_example().await?;

    Ok(())
}

async fn basic_nested_flow_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Example 1: Basic Nested Flow");
    println!("--------------------------------");

    // Create a simple validation flow
    let validation_flow = FlowBuilder::new()
        .start_node("validate")
        .node(
            "validate",
            Node::new(SetValueNode::new(
                "validation_result".to_string(),
                json!("validated"),
                Action::simple("complete"),
            )),
        )
        .build();

    // Create a processing flow
    let processing_flow = FlowBuilder::new()
        .start_node("process")
        .node(
            "process",
            Node::new(SetValueNode::new(
                "processing_result".to_string(),
                json!("processed"),
                Action::simple("complete"),
            )),
        )
        .build();

    // Create main flow that uses both sub-flows
    let mut main_flow = FlowBuilder::new()
        .start_node("start")
        .node(
            "start",
            Node::new(SetValueNode::new(
                "input_data".to_string(),
                json!("Hello World!"),
                Action::simple("to_validation"),
            )),
        )
        .node("validation", Node::new(FlowNode::new(validation_flow)))
        .node("processing", Node::new(FlowNode::new(processing_flow)))
        .node(
            "finish",
            Node::new(SetValueNode::new(
                "final_result".to_string(),
                json!("completed"),
                Action::simple("done"),
            )),
        )
        .route("start", "to_validation", "validation")
        .route("validation", "complete", "processing")
        .route("processing", "complete", "finish")
        .build();

    // Execute the main flow
    let mut store = SharedStore::new();
    let result = main_flow.execute(&mut store).await?;

    println!("✅ Flow executed successfully!");
    println!("📊 Steps executed: {}", result.steps_executed);
    println!("🛤️  Execution path: {:?}", result.execution_path);

    // Show the processed data
    let input_data = store.get("input_data")?.unwrap();
    println!("📄 Input data: {}", input_data);

    let validation_result = store.get("validation_result")?.unwrap();
    println!("✅ Validation result: {}", validation_result);

    let processing_result = store.get("processing_result").unwrap();
    if processing_result.is_some() {
        println!("⚙️  Processing result: {}", processing_result.unwrap());
    } else {
        println!("⚙️  Processing result: Not reached (flow terminated after validation)");
    }

    Ok(())
}

async fn data_processing_pipeline_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏭 Example 2: Data Processing Pipeline");
    println!("--------------------------------------");

    // Create ETL sub-flows
    let extract_flow = FlowBuilder::new()
        .start_node("extract")
        .node(
            "extract",
            Node::new(SetValueNode::new(
                "raw_data".to_string(),
                json!(["record1", "record2", "record3"]),
                Action::simple("complete"),
            )),
        )
        .build();

    let transform_flow = FlowBuilder::new()
        .start_node("transform")
        .node(
            "transform",
            Node::new(SetValueNode::new(
                "transformed_data".to_string(),
                json!(["RECORD1", "RECORD2", "RECORD3"]),
                Action::simple("complete"),
            )),
        )
        .build();

    let load_flow = FlowBuilder::new()
        .start_node("load")
        .node(
            "load",
            Node::new(SetValueNode::new(
                "loaded_records".to_string(),
                json!(3),
                Action::simple("complete"),
            )),
        )
        .build();

    // Create main ETL pipeline
    let mut etl_pipeline = FlowBuilder::new()
        .start_node("extract_phase")
        .node("extract_phase", Node::new(FlowNode::new(extract_flow)))
        .node("transform_phase", Node::new(FlowNode::new(transform_flow)))
        .node("load_phase", Node::new(FlowNode::new(load_flow)))
        .route("extract_phase", "complete", "transform_phase")
        .route("transform_phase", "complete", "load_phase")
        .build();

    // Execute the ETL pipeline
    let mut store = SharedStore::new();
    let result = etl_pipeline.execute(&mut store).await?;

    println!("✅ ETL Pipeline executed successfully!");
    println!("📊 Total steps: {}", result.steps_executed);

    let raw_data = store.get("raw_data")?.unwrap();
    println!("📥 Raw data: {}", raw_data);

    let transformed_data = store.get("transformed_data").unwrap();
    if transformed_data.is_some() {
        println!("🔄 Transformed data: {}", transformed_data.unwrap());
    } else {
        println!("🔄 Transformed data: Not reached");
    }

    let loaded_records = store.get("loaded_records").unwrap();
    if loaded_records.is_some() {
        println!("📈 Records loaded: {}", loaded_records.unwrap());
    } else {
        println!("📈 Records loaded: Not reached");
    }

    Ok(())
}
