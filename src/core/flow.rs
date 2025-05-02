use crate::core::communication::{Params, ParamsContainer, SharedStore};
use crate::core::node::Node;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Flow orchestrates a graph of nodes with action-based transitions
pub struct Flow<P, E> {
    start_node: Option<Arc<dyn Node<PrepResult = P, ExecResult = E>>>,
    transitions: HashMap<String, Arc<dyn Node<PrepResult = P, ExecResult = E>>>,
    params: Params,
}

impl<P, E> Flow<P, E>
where
    P: Send + Default + Clone + 'static,
    E: Send + Default + 'static,
{
    /// Create a new Flow with an optional start node
    pub fn new(start_node: Option<Arc<dyn Node<PrepResult = P, ExecResult = E>>>) -> Self {
        Self {
            start_node,
            transitions: HashMap::new(),
            params: Params::new(),
        }
    }

    /// Set the start node for this flow
    pub fn set_start(&mut self, node: Arc<dyn Node<PrepResult = P, ExecResult = E>>) {
        self.start_node = Some(node);
    }

    /// Add a transition: action -> next node
    pub fn add_transition(
        &mut self,
        action: &str,
        node: Arc<dyn Node<PrepResult = P, ExecResult = E>>,
    ) {
        self.transitions.insert(action.to_string(), node);
    }

    /// Set parameters for this flow
    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    /// Run the flow with given shared store, following action-based transitions
    pub async fn run(&self, shared: &SharedStore) -> Result<Option<String>> {
        let mut current = match &self.start_node {
            Some(node) => node.clone(),
            None => return Ok(None),
        };

        loop {
            // set Flow params to current node
            // current.set_params(self.params.clone());

            let action = current.run(shared).await?;

            match action {
                Some(ref act) => {
                    if let Some(next) = self.transitions.get(act) {
                        current = next.clone();
                    } else {
                        // 没有下一个节点，流程结束
                        return Ok(Some(act.clone()));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    pub fn start_node(&self) -> &Option<Arc<dyn Node<PrepResult = P, ExecResult = E>>> {
        &self.start_node
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn transitions(&self) -> &HashMap<String, Arc<dyn Node<PrepResult = P, ExecResult = E>>> {
        &self.transitions
    }
}

/// Adapter to use a Flow as a Node if needed
pub struct FlowNodeAdapter<P, E> {
    flow: Flow<P, E>,
}

impl<P, E> FlowNodeAdapter<P, E> {
    pub fn new(flow: Flow<P, E>) -> Self {
        Self { flow }
    }
}

#[async_trait::async_trait]
impl<P, E> Node for FlowNodeAdapter<P, E>
where
    P: Send + Default + Clone + 'static,
    E: Send + Default + 'static,
{
    type PrepResult = ();
    type ExecResult = Option<String>;

    async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
        self.flow.run(&SharedStore::default()).await
    }

    async fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: Self::PrepResult,
        exec_res: Self::ExecResult,
    ) -> Result<Option<String>> {
        Ok(exec_res)
    }

    async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
        Ok(())
    }
}

impl<P, E> ParamsContainer for FlowNodeAdapter<P, E> {
    fn set_params(&mut self, params: Params) {
        self.flow.params = params
    }

    fn get_params(&self) -> &Params {
        &self.flow.params
    }

    fn get_params_mut(&mut self) -> &mut Params {
        &mut self.flow.params
    }
}
