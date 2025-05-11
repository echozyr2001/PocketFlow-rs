use anyhow::Result;
use async_trait::async_trait;
use pocketflow_rs::{
    communication::{BaseSharedStore, SharedStore},
    core::{ExecResult, PostResult, PrepResult, flow::Flow, node::NodeTrait},
    node::BaseNode,
};
use serde_json::Value as JsonValue;
use std::sync::Arc; // For Node3

// ------------------------------------
// 1. Simple Flow: action transition
// ------------------------------------

struct GoNode {
    base: BaseNode,
}
#[async_trait]
impl NodeTrait for GoNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.base.prep(shared_store)
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.base.exec(prep_res)
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("go"))
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store)
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post(shared_store, prep_res, exec_res)
    }
}

struct EndNode {
    base: BaseNode,
}

#[async_trait]
impl NodeTrait for EndNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.base.prep(shared_store)
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.base.exec(prep_res)
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("end")) // This is the final action
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store)
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post(shared_store, prep_res, exec_res)
    }
    // get_successor returns None by default
}

#[test]
fn test_flow_action_transition() {
    let end_node_arc = Arc::new(EndNode {
        base: BaseNode::new(),
    });
    let go_node_arc = Arc::new(GoNode {
        base: BaseNode::new(),
    });

    let shared = BaseSharedStore::new_in_memory();
    let mut flow = Flow::new(Some(go_node_arc));
    flow.add_transition("go".into(), end_node_arc.clone()); // This is the old way, now removed
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
    base: BaseNode,
}
#[async_trait]
impl NodeTrait for Node1 {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.base.prep(shared_store)
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.base.exec(prep_res)
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("next"))
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store)
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post(shared_store, prep_res, exec_res)
    }
}

struct Node2 {
    base: BaseNode,
}

#[async_trait]
impl NodeTrait for Node2 {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.base.prep(shared_store)
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.base.exec(prep_res)
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("done")) // This is the final action of the inner flow
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store)
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post(shared_store, prep_res, exec_res)
    }
}

struct Node3 {
    base: BaseNode,
}

#[async_trait]
impl NodeTrait for Node3 {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.base.prep(shared_store)
    }

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

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store)
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post(shared_store, prep_res, exec_res)
    }
}

#[test]
fn test_flow_nested() {
    let n2_arc = Arc::new(Node2 {
        base: BaseNode::new(),
    });
    let n1_arc = Arc::new(Node1 {
        base: BaseNode::new(),
    });

    // Inner flow definition
    let mut inner_flow = Flow::new(Some(n1_arc));
    inner_flow.add_transition("next".into(), n2_arc.clone()); // This was the old way
    // No add_transition for inner_flow, Node1 handles its successor.

    let n3_arc = Arc::new(Node3 {
        base: BaseNode::new(),
    });

    let inner_flow_arc = Arc::new(inner_flow);

    let mut outer = Flow::new(Some(inner_flow_arc));
    outer.add_transition("done".into(), n3_arc.clone()); // This was the old way
    // outer.add_transition("done", n3_arc.clone()); // This was the old way

    let shared = BaseSharedStore::new_in_memory();
    let result = outer.run(&shared).unwrap(); // outer.run will execute wrapped_inner_flow.run, which runs inner_flow.
    // inner_flow finishes with "done".
    // wrapped_inner_flow.get_successor("done") returns n3_arc.
    // outer flow continues with n3_arc.
    // n3_arc runs, its post returns "outer_done".
    // n3_arc.get_successor("outer_done") is None, so flow ends.
    assert_eq!(result, PostResult::from("outer_done"));
}
