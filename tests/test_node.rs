use pocketflow_rs::{
    communication::{Params, ParamsContainer},
    node::BaseNode,
};

#[tokio::test]
async fn test_basenode_params() {
    let mut node: BaseNode<(), ()> = BaseNode::new(1, 0);
    let mut params = Params::new();
    params.set("foo", "bar");
    node.set_params(params.clone());
    assert_eq!(
        node.get_params().get::<String>("foo"),
        Some("bar".to_string())
    );
}
