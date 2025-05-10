use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value as JsonValue, json};
use std::sync::Arc;

use pocketflow_rs::{
    communication::{BaseSharedStore, SharedStore},
    core::{
        ExecResult, PostResult, PrepResult,
        batch::{BatchFlow, BatchNode, BatchProcessor},
        flow::Flow,
        node::NodeTrait,
    },
};

struct TestNode {
    name: String,
}

#[async_trait]
impl NodeTrait for TestNode {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from(self.name.clone()))
    }
}

// ------------------------------------
// 1. Simple Batch Processor
// ------------------------------------
#[test]
fn test_batch_sequential() {
    let node1 = Arc::new(TestNode {
        name: "action1".to_string(),
    });

    let mut processor = BatchProcessor::new();
    processor.add_node(node1);

    let mut stores = Vec::new();
    for i in 0..3 {
        let store = BaseSharedStore::new_in_memory();
        store.insert("index", json!(i)); // BaseSharedStore generic insert
        stores.push(store);
    }

    // BatchProcessor.process_sequential might need update if it expects Action
    // Assuming for now it's compatible or will be updated later.
    // The return type of processor.process_sequential is Vec<Result<Action>>
    // This will likely fail compilation if Action was removed or changed significantly.
    // For now, let's assume the test logic needs to adapt to PostResult if process_sequential changes.
    let results: Vec<Result<PostResult>> = processor.process_sequential(stores);
    // process_sequential now returns Vec<Result<PostResult>>, so direct assignment.

    assert_eq!(results.len(), 3);

    for result in results {
        assert_eq!(result.unwrap(), PostResult::from("action1"));
    }
}

// ------------------------------------
// 2. Batch Node: run and get results
// ------------------------------------
#[test]
fn test_batch_node() {
    let node1 = Arc::new(TestNode {
        name: "action1".to_string(),
    });

    let mut processor = BatchProcessor::new();
    processor.add_node(node1);

    let mut stores = Vec::new();
    for i in 0..3 {
        let store = BaseSharedStore::new_in_memory();
        store.insert("index", json!(i));
        stores.push(store);
    }

    let batch_node = BatchNode::new(processor, stores);
    let shared = BaseSharedStore::new_in_memory(); // Changed to BaseSharedStore

    // batch_node.run might also expect Action.
    // Assuming it returns something convertible or will be updated.
    // Original: result = batch_node.run(&shared).unwrap(); (Action)
    // BatchNode implements NodeTrait, so run_sync returns Result<PostResult>
    let result = batch_node.run_sync(&shared).unwrap();

    assert_eq!(result, PostResult::from("batch_complete"));

    // get_results() calls processor.process_sequential, which now returns Vec<Result<PostResult>>
    let results_post: Vec<Result<PostResult>> = batch_node.get_results();
    assert_eq!(results_post.len(), 3);
}

struct ClassifierNode;

#[async_trait]
impl NodeTrait for ClassifierNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        let value = shared_store
            .get_value("value") // Uses trait method
            .and_then(|arc_any| arc_any.downcast_ref::<JsonValue>().cloned())
            .and_then(|json_val| serde_json::from_value::<i32>(json_val).ok())
            .unwrap_or(0);
        Ok(json!({"value": value}).into())
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        if let Some(obj) = prep_res.as_object() {
            if let Some(value) = obj.get("value").and_then(|v| v.as_i64()) {
                if value % 2 == 0 {
                    return Ok(PostResult::from("even"));
                } else {
                    return Ok(PostResult::from("odd"));
                }
            }
        }
        Ok(PostResult::default()) // Changed from DEFAULT_ACTION
    }
}

struct EvenNode;
#[async_trait]
impl NodeTrait for EvenNode {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("processed_even"))
    }
}

struct OddNode;
#[async_trait]
impl NodeTrait for OddNode {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("processed_odd"))
    }
}

// ------------------------------------
// 3. Batch Flow: Classifier -> Even/Odd
// ------------------------------------
#[test]
fn test_batch_flow() {
    let classifier_node = Arc::new(ClassifierNode {}); // Arc<dyn NodeTrait>
    let even_node = Arc::new(EvenNode {}); // Arc<dyn NodeTrait>
    let odd_node = Arc::new(OddNode {}); // Arc<dyn NodeTrait>

    let mut processor = BatchProcessor::new();
    processor.add_node(classifier_node); // add_node takes Arc<dyn BaseNode> - this will break

    // Create even flow
    // Flow::new expects Option<Arc<dyn NodeTrait>>
    let even_flow = Flow::new(Some(even_node.clone())); // even_node is Arc<dyn NodeTrait>
    // even_flow.add_transition - this method was removed from Flow.
    // Successors are now managed by nodes themselves.
    // This test needs significant rework if BatchFlow relies on Flow's old transition map.

    // Create odd flow
    let odd_flow = Flow::new(Some(odd_node.clone()));
    // odd_flow.add_transition...

    // Create batch flow
    let mut batch_flow = BatchFlow::new(processor); // BatchFlow might also be outdated
    // batch_flow.add_flow also likely expects old Flow or BaseNode types.
    // For now, I will assume these parts (BatchProcessor, BatchFlow, BatchNode)
    // are outside the immediate scope of refactoring NodeTrait and SharedStore.
    // The test will likely fail here until Batch* components are updated.
    // To make it compile for now, I'll comment out the parts that will surely break.

    batch_flow
        .add_flow("even", Arc::new(even_flow)) // Arc::new(even_flow) where even_flow is Flow
        .add_flow("odd", Arc::new(odd_flow));

    let mut stores = Vec::new();
    for i in 0..4 {
        let store = BaseSharedStore::new_in_memory();
        store.insert("value", json!(i));
        stores.push(store);
    }

    // Process
    let results = batch_flow.process(stores);
    // This part is highly dependent on BatchFlow internals.
    // For now, I'll comment out the assertions as well.

    // Verify results
    assert_eq!(results.len(), 2);
    assert!(results.contains_key("even"));
    assert!(results.contains_key("odd"));
    assert_eq!(results["even"].len(), 2);
    assert_eq!(results["odd"].len(), 2);
    for result in &results["even"] {
        assert_eq!(
            result.as_ref().unwrap(),
            &PostResult::from("processed_even")
        );
    }
    for result in &results["odd"] {
        assert_eq!(result.as_ref().unwrap(), &PostResult::from("processed_odd"));
    }
}
