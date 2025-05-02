use anyhow::Result;
use async_trait::async_trait;
use pocketflow_rs::{
    communication::{Params, ParamsContainer, SharedStore},
    core::flow::{Flow, FlowNodeAdapter},
    node::Node,
};
use std::sync::{Arc, OnceLock};

fn get_empty_params() -> &'static Params {
    static EMPTY: OnceLock<Params> = OnceLock::new();
    EMPTY.get_or_init(Params::new)
}

#[tokio::test]
async fn test_flow_action_transition() {
    struct GoNode;
    #[async_trait]
    impl Node for GoNode {
        type PrepResult = ();
        type ExecResult = ();
        async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
            Ok(())
        }
        async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
            Ok(())
        }
        async fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: Self::PrepResult,
            _exec_res: Self::ExecResult,
        ) -> Result<Option<String>> {
            Ok(Some("go".to_string()))
        }
    }
    impl ParamsContainer for GoNode {
        fn set_params(&mut self, _params: Params) {}
        fn get_params(&self) -> &Params {
            get_empty_params()
        }
        fn get_params_mut(&mut self) -> &mut Params {
            // For tests, you can panic or return a dummy mutable reference if needed
            panic!("get_params_mut not supported for stateless test node");
        }
    }
    struct EndNode;
    #[async_trait]
    impl Node for EndNode {
        type PrepResult = ();
        type ExecResult = ();
        async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
            Ok(())
        }
        async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
            Ok(())
        }
        async fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: Self::PrepResult,
            _exec_res: Self::ExecResult,
        ) -> Result<Option<String>> {
            Ok(Some("end".to_string()))
        }
    }
    impl ParamsContainer for EndNode {
        fn set_params(&mut self, _params: Params) {}
        fn get_params(&self) -> &Params {
            get_empty_params()
        }
        fn get_params_mut(&mut self) -> &mut Params {
            // For tests, you can panic or return a dummy mutable reference if needed
            panic!("get_params_mut not supported for stateless test node");
        }
    }
    let go = Arc::new(GoNode);
    let end = Arc::new(EndNode);
    let shared = SharedStore::new();
    let mut flow = Flow::<(), ()>::new(Some(go.clone()));
    flow.add_transition("go", end.clone());
    let result = flow.run(&shared).await.unwrap();
    assert_eq!(result, Some("end".to_string()));
}

#[tokio::test]
async fn test_flow_nested() {
    // inner flow: node1 -> node2
    struct Node1;
    #[async_trait]
    impl Node for Node1 {
        type PrepResult = ();
        type ExecResult = ();
        async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
            Ok(())
        }
        async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
            Ok(())
        }
        async fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: Self::PrepResult,
            _exec_res: Self::ExecResult,
        ) -> Result<Option<String>> {
            Ok(Some("next".to_string()))
        }
    }
    impl ParamsContainer for Node1 {
        fn set_params(&mut self, _params: Params) {}
        fn get_params(&self) -> &Params {
            get_empty_params()
        }
        fn get_params_mut(&mut self) -> &mut Params {
            // For tests, you can panic or return a dummy mutable reference if needed
            panic!("get_params_mut not supported for stateless test node");
        }
    }
    struct Node2;
    #[async_trait]
    impl Node for Node2 {
        type PrepResult = ();
        type ExecResult = ();
        async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
            Ok(())
        }
        async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
            Ok(())
        }
        async fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: Self::PrepResult,
            _exec_res: Self::ExecResult,
        ) -> Result<Option<String>> {
            Ok(Some("done".to_string()))
        }
    }
    impl ParamsContainer for Node2 {
        fn set_params(&mut self, _params: Params) {}
        fn get_params(&self) -> &Params {
            get_empty_params()
        }
        fn get_params_mut(&mut self) -> &mut Params {
            // For tests, you can panic or return a dummy mutable reference if needed
            panic!("get_params_mut not supported for stateless test node");
        }
    }
    let n1 = Arc::new(Node1);
    let n2 = Arc::new(Node2);
    let mut inner = Flow::<(), ()>::new(Some(n1.clone()));
    inner.add_transition("next", n2.clone());
    // 外层 flow: inner_flow -> node3
    struct Node3;
    #[async_trait]
    impl Node for Node3 {
        type PrepResult = ();
        type ExecResult = Option<String>;
        async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
            Ok(())
        }
        async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
            Ok(Some("outer_done".to_string()))
        }
        async fn post(
            &self,
            _shared: &SharedStore,
            _prep_res: Self::PrepResult,
            exec_res: Self::ExecResult,
        ) -> Result<Option<String>> {
            Ok(exec_res)
        }
    }
    impl ParamsContainer for Node3 {
        fn set_params(&mut self, _params: Params) {}
        fn get_params(&self) -> &Params {
            get_empty_params()
        }
        fn get_params_mut(&mut self) -> &mut Params {
            panic!()
        }
    }
    let node3 = Arc::new(Node3);
    let mut outer = Flow::<(), Option<String>>::new(Some(Arc::new(FlowNodeAdapter::new(inner))));
    outer.add_transition("done", node3.clone());
    let shared = SharedStore::new();
    let result = outer.run(&shared).await.unwrap();
    assert_eq!(result, Some("outer_done".to_string()));
}
