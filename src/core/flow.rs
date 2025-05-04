use crate::core::communication::{Params, SharedStore};
use crate::core::node::BaseNode;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::{Action, ExecResult, NONE_ACTION, PrepResult};

/// Flow orchestrates a graph of nodes with action-based transitions
pub struct Flow {
    start_node: Option<Arc<dyn BaseNode>>,
    transitions: HashMap<String, Arc<dyn BaseNode>>,
    params: Params,
}

impl Flow {
    /// Create a new Flow with an optional start node
    pub fn new(start_node: Option<Arc<dyn BaseNode>>) -> Self {
        Self {
            start_node,
            transitions: HashMap::new(),
            params: Params::new(),
        }
    }

    /// Add a transition: action -> next node
    pub fn add_transition(&mut self, action: &str, node: Arc<dyn BaseNode>) {
        self.transitions.insert(action.to_string(), node);
    }

    /// Run the flow with given shared store, following action-based transitions
    fn _run_flow(&self, shared: &SharedStore) -> Result<Action> {
        let mut current = match &self.start_node {
            Some(node) => node.clone(),
            None => return Ok(NONE_ACTION),
        };

        loop {
            // set Flow params to current node
            // current.set_params(self.params.clone());

            let action = current.run(shared)?;

            match action.0 {
                Some(act) => {
                    if let Some(next) = self.transitions.get(act) {
                        current = next.clone();
                    } else {
                        // 没有下一个节点，流程结束
                        return Ok(act.to_owned().into());
                    }
                }
                None => return Ok(NONE_ACTION),
            }
        }
    }
}

impl Flow {
    /// Set parameters for this flow
    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    /// Get the start node of this flow
    pub fn start_node(&self) -> &Option<Arc<dyn BaseNode>> {
        &self.start_node
    }

    /// Get the transitions of this flow
    pub fn transitions(&self) -> &HashMap<String, Arc<dyn BaseNode>> {
        &self.transitions
    }

    /// Get the parameters of this flow
    pub fn params(&self) -> &Params {
        &self.params
    }
}

impl BaseNode for Flow {
    fn post(
        &self,
        shared: &SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        // When Flow is used as a node, run the flow itself
        // and return the resulting action
        self._run_flow(shared)
    }
}
