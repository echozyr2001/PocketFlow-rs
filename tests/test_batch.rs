use std::sync::Arc;

use pocketflow_rs::{
    communication::SharedStore,
    core::{
        Action, DEFAULT_ACTION, ExecResult, PrepResult, Result,
        batch::{BatchFlow, BatchNode, BatchProcessor},
        flow::Flow,
    },
    node::BaseNode,
};

struct TestNode {
    name: String,
}

impl BaseNode for TestNode {
    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        Ok(self.name.clone().into())
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
        let store = SharedStore::new();
        store.insert("index", serde_json::json!(i));
        stores.push(store);
    }

    let results = processor.process_sequential(stores);
    assert_eq!(results.len(), 3);

    for result in results {
        assert_eq!(result.unwrap(), "action1".into());
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
        let store = SharedStore::new();
        store.insert("index", serde_json::json!(i));
        stores.push(store);
    }

    let batch_node = BatchNode::new(processor, stores);
    let shared = SharedStore::new();

    let result = batch_node.run(&shared).unwrap();
    assert_eq!(result, "batch_complete".into());

    let results = batch_node.get_results();
    assert_eq!(results.len(), 3);
}

struct ClassifierNode;

impl BaseNode for ClassifierNode {
    fn prep(&self, shared: &SharedStore) -> Result<PrepResult> {
        let value = shared
            .get::<serde_json::Value>("value")
            .and_then(|json_val| serde_json::from_value::<i32>(json_val).ok())
            .unwrap_or(0);
        Ok(serde_json::json!({"value": value}).into())
    }

    fn post(
        &self,
        _shared: &SharedStore,
        prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        if let Some(obj) = prep_res.as_object() {
            if let Some(value) = obj.get("value").and_then(|v| v.as_i64()) {
                if value % 2 == 0 {
                    return Ok("even".into());
                } else {
                    return Ok("odd".into());
                }
            }
        }

        Ok(DEFAULT_ACTION)
    }
}

struct EvenNode;

impl BaseNode for EvenNode {
    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        Ok("processed_even".into())
    }
}

struct OddNode;

impl BaseNode for OddNode {
    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        Ok("processed_odd".into())
    }
}

// ------------------------------------
// 3. Batch Flow: Classifier -> Even/Odd
// ------------------------------------
#[test]
fn test_batch_flow() {
    // Create classifier
    let classifier = Arc::new(ClassifierNode);

    let mut processor = BatchProcessor::new();
    processor.add_node(classifier);

    // Create even flow
    let even_flow = Flow::new(Some(Arc::new(EvenNode)));

    // Create odd flow
    let odd_flow = Flow::new(Some(Arc::new(OddNode)));

    // Create batch flow
    let mut batch_flow = BatchFlow::new(processor);
    batch_flow
        .add_flow("even", Arc::new(even_flow))
        .add_flow("odd", Arc::new(odd_flow));

    // Create test data
    let mut stores = Vec::new();
    for i in 0..4 {
        let store = SharedStore::new();
        store.insert("value", serde_json::json!(i));
        stores.push(store);
    }

    // Process
    let results = batch_flow.process(stores);
    // Verify results
    assert_eq!(results.len(), 2); // even and odd groups

    assert!(results.contains_key("even"));
    assert!(results.contains_key("odd"));

    assert_eq!(results["even"].len(), 2); // 0, 2
    assert_eq!(results["odd"].len(), 2); // 1, 3

    // Check all even results
    for result in &results["even"] {
        assert_eq!(result.as_ref().unwrap(), &"processed_even".into());
    }

    // Check all odd results
    for result in &results["odd"] {
        assert_eq!(result.as_ref().unwrap(), &"processed_odd".into());
    }
}
