use std::sync::Arc;

use anyhow::Result;
use pocketflow_rs::{
    communication::SharedStore,
    core::{Action, ExecResult, PrepResult, flow::Flow},
    node::BaseNode,
};

// ------------------------------------
// 1. Simple Flow: action transition
// ------------------------------------
#[test]
fn test_flow_action_transition() {
    struct GoNode;
    impl BaseNode for GoNode {
        fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: &PrepResult,
            _exec_res: &ExecResult,
        ) -> Result<Action> {
            Ok("go".into())
        }
    }

    struct EndNode;
    impl BaseNode for EndNode {
        fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: &PrepResult,
            _exec_res: &ExecResult,
        ) -> Result<Action> {
            Ok("end".into())
        }
    }

    let go = Arc::new(GoNode);
    let end = Arc::new(EndNode);
    let shared = SharedStore::new_in_memory();
    let mut flow = Flow::new(Some(go));
    flow.add_transition("go", end);
    let result = flow.run(&shared);
    match result {
        Ok(action) => assert_eq!(action, "end".into()),
        Err(e) => panic!("Expected success, but got error: {}", e),
    }
}

// ------------------------------------
// 2. Nested Flow: (node1 -> node2) -> node3
// ------------------------------------
#[test]
fn test_flow_nested() {
    // inner flow: node1 -> node2
    struct Node1;
    impl BaseNode for Node1 {
        fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: &PrepResult,
            _exec_res: &ExecResult,
        ) -> Result<Action> {
            Ok("next".into())
        }
    }
    struct Node2;
    impl BaseNode for Node2 {
        fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: &PrepResult,
            _exec_res: &ExecResult,
        ) -> Result<Action> {
            Ok("done".into())
        }
    }

    let n1 = Arc::new(Node1);
    let n2 = Arc::new(Node2);
    let mut inner = Flow::new(Some(n1));
    inner.add_transition("next", n2);

    // outer flow: inner_flow -> node3
    struct Node3;
    impl BaseNode for Node3 {
        fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
            Ok(serde_json::Value::String("outer_done".into()).into())
        }
        fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: &PrepResult,
            exec_res: &ExecResult,
        ) -> Result<Action> {
            if let Some(s) = exec_res.as_str() {
                Ok(s.to_owned().into())
            } else {
                Err(anyhow::anyhow!("Expected string in prep_res"))
            }
        }
    }

    let node3 = Arc::new(Node3);
    let mut outer = Flow::new(Some(Arc::new(inner)));
    outer.add_transition("done", node3.clone());
    let shared = SharedStore::new_in_memory();
    let result = outer.run(&shared).unwrap();
    assert_eq!(result, "outer_done".into());
}
