use anyhow::Result;
use async_trait::async_trait;
use pocketflow_rs::{
    communication::{BaseSharedStore, SharedStore},
    core::{ExecResult, PostResult, PrepResult, flow::Flow, node::NodeTrait},
};
use serde_json::Value as JsonValue;
use std::sync::Arc; // For Node3

// ------------------------------------
// 1. Simple Flow: action transition
// ------------------------------------

struct GoNode {
    next_node: Option<Arc<dyn NodeTrait>>,
}
#[async_trait]
impl NodeTrait for GoNode {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("go"))
    }
    fn get_successor(&self, action: &str) -> Option<Arc<dyn NodeTrait>> {
        if action == "go" {
            self.next_node.clone()
        } else {
            None
        }
    }
}

struct EndNode;
#[async_trait]
impl NodeTrait for EndNode {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("end")) // This is the final action
    }
    // get_successor returns None by default
}

#[test]
fn test_flow_action_transition() {
    let end_node_arc = Arc::new(EndNode);
    let go_node_arc = Arc::new(GoNode {
        next_node: Some(end_node_arc),
    });

    let shared = BaseSharedStore::new_in_memory();
    let flow = Flow::new(Some(go_node_arc));
    // flow.add_transition is removed. Successors are part of nodes.

    let result = flow.run(&shared); // Flow::run now uses node.get_successor
    match result {
        Ok(post_res) => assert_eq!(post_res, PostResult::from("end")),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 2. Nested Flow: (node1 -> node2) -> node3
// ------------------------------------

struct Node1 {
    next_node: Option<Arc<dyn NodeTrait>>,
}
#[async_trait]
impl NodeTrait for Node1 {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("next"))
    }
    fn get_successor(&self, action: &str) -> Option<Arc<dyn NodeTrait>> {
        if action == "next" {
            self.next_node.clone()
        } else {
            None
        }
    }
}

struct Node2;
#[async_trait]
impl NodeTrait for Node2 {
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("done")) // This is the final action of the inner flow
    }
}

struct Node3;
#[async_trait]
impl NodeTrait for Node3 {
    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        Ok(JsonValue::String("outer_done".into()).into())
    }
    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        if let Some(s) = exec_res.as_str() {
            Ok(PostResult::from(s.to_owned()))
        } else {
            Err(anyhow::anyhow!("Expected string in exec_res for Node3"))
        }
    }
}

#[test]
fn test_flow_nested() {
    let n2_arc = Arc::new(Node2);
    let n1_arc = Arc::new(Node1 {
        next_node: Some(n2_arc),
    });

    // Inner flow definition
    let inner_flow = Flow::new(Some(n1_arc));
    // No add_transition for inner_flow, Node1 handles its successor.

    let n3_arc = Arc::new(Node3);

    // To make the outer flow transition from inner_flow to Node3 on "done",
    // the `inner_flow` (when treated as a NodeTrait) needs to handle get_successor("done").
    // This requires `Flow` to implement `add_successor` and `get_successor` meaningfully.
    // For now, `Flow`'s default `get_successor` returns `None`.
    // This test will need `Flow` to be enhanced or the test structure rethought.

    // Let's create a wrapper node for the inner flow that handles the "done" transition.
    struct FlowWrapperNode {
        flow_to_run: Arc<Flow>,
        done_successor: Option<Arc<dyn NodeTrait>>,
    }

    #[async_trait]
    impl NodeTrait for FlowWrapperNode {
        fn run_sync(&self, shared_store: &dyn SharedStore) -> Result<PostResult> {
            self.flow_to_run.run(shared_store)
        }
        async fn run_async(&self, shared_store: &dyn SharedStore) -> Result<PostResult> {
            self.flow_to_run.run_async(shared_store).await
        }
        fn get_successor(&self, action: &str) -> Option<Arc<dyn NodeTrait>> {
            if action == "done" {
                self.done_successor.clone()
            } else {
                None
            }
        }
        // Implement other NodeTrait methods with defaults or by delegation if needed
    }

    let inner_flow_arc = Arc::new(inner_flow);
    let wrapped_inner_flow = Arc::new(FlowWrapperNode {
        flow_to_run: inner_flow_arc,
        done_successor: Some(n3_arc),
    });

    let outer = Flow::new(Some(wrapped_inner_flow));
    // outer.add_transition("done", n3_arc.clone()); // This was the old way

    let shared = BaseSharedStore::new_in_memory();
    let result = outer.run(&shared).unwrap(); // outer.run will execute wrapped_inner_flow.run_sync, which runs inner_flow.
    // inner_flow finishes with "done".
    // wrapped_inner_flow.get_successor("done") returns n3_arc.
    // outer flow continues with n3_arc.
    // n3_arc runs, its post returns "outer_done".
    // n3_arc.get_successor("outer_done") is None, so flow ends.
    assert_eq!(result, PostResult::from("outer_done"));
}
