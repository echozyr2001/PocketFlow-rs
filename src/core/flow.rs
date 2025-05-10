use crate::core::{
    communication::{Params, SharedStore},
    node::NodeTrait,
    r#type::{ExecResult, PostResult, PrepResult},
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
// HashMap might not be needed if transitions are fully on nodes.
// use std::collections::HashMap;

/// Flow orchestrates a graph of nodes by following successor logic defined within nodes.
pub struct Flow {
    start_node: Option<Arc<dyn NodeTrait>>,
    // transitions: HashMap<String, Arc<dyn NodeTrait>>, // Replaced by NodeTrait::get_successor
    params: Params, // Assuming Params is still relevant for a Flow's context
}

impl Flow {
    /// Create a new Flow with an optional start node.
    pub fn new(start_node: Option<Arc<dyn NodeTrait>>) -> Self {
        Self {
            start_node,
            // transitions: HashMap::new(),
            params: Params::new(), // Flows can have their own params
        }
    }

    // add_transition is removed. Transitions are managed by nodes via NodeTrait::add_successor.
    // If Flow needs to help wire nodes, it would be done externally before passing start_node.

    /// Run the flow synchronously.
    /// The flow proceeds by calling `run` on the current node,
    /// then using `get_successor` on that same node with the `PostResult`
    /// to determine the next node.
    /// The flow terminates when a node has no successor for the given `PostResult`,
    /// returning the last `PostResult`.
    pub fn run(&self, shared_store: &dyn SharedStore) -> Result<PostResult> {
        let mut current_node_opt = self.start_node.clone();
        let mut last_post_result = PostResult::default(); // Default if flow doesn't start

        while let Some(current_node) = current_node_opt {
            // TODO: How are flow-level params applied to nodes if needed?
            // Current NodeTrait methods don't take Params directly.
            // Nodes might access params from SharedStore if they are put there.

            let post_result = current_node.run(shared_store)?;
            last_post_result = post_result.clone(); // Clone for the loop and potential return

            if let Some(action_str) = last_post_result.as_str().non_empty_or_none() {
                current_node_opt = current_node.get_successor(action_str);
            } else {
                // Empty PostResult string, or specific "end" signal might mean flow termination.
                // Or, get_successor returning None is the only termination condition.
                current_node_opt = None;
            }

            if current_node_opt.is_none() {
                // No successor, flow ends. Return the last PostResult.
                return Ok(last_post_result);
            }
        }

        // If start_node was None, or loop finishes without returning (e.g. empty PostResult and no successor)
        Ok(last_post_result)
    }

    /// Run the flow asynchronously.
    pub async fn run_async(&self, shared_store: &dyn SharedStore) -> Result<PostResult> {
        let mut current_node_opt = self.start_node.clone();
        let mut last_post_result = PostResult::default();

        while let Some(current_node) = current_node_opt {
            let post_result = current_node.run_async(shared_store).await?;
            last_post_result = post_result.clone();

            if let Some(action_str) = last_post_result.as_str().non_empty_or_none() {
                current_node_opt = current_node.get_successor(action_str);
            } else {
                current_node_opt = None;
            }

            if current_node_opt.is_none() {
                return Ok(last_post_result);
            }
        }
        Ok(last_post_result)
    }

    /// Set parameters for this flow.
    pub fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    /// Get the parameters of this flow.
    pub fn params(&self) -> &Params {
        &self.params
    }
}

// Helper trait for &str to simplify flow logic
trait NonEmptyOrNone {
    fn non_empty_or_none(&self) -> Option<&str>;
}

impl NonEmptyOrNone for &str {
    fn non_empty_or_none(&self) -> Option<&str> {
        if self.is_empty() { None } else { Some(self) }
    }
}

// Flow itself can be a NodeTrait, allowing nested flows.
#[async_trait]
impl NodeTrait for Flow {
    // prep, exec for a Flow node could initialize its own context or be no-ops
    // if its execution is solely defined by its internal start_node.
    fn prep(&self, _shared_store: &dyn SharedStore) -> Result<PrepResult> {
        // A Flow as a node might not have specific prep, or it might set up
        // its internal shared_store context using its self.params.
        // For now, default.
        Ok(PrepResult::default())
    }

    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        // Executing a Flow as a node means running its internal logic.
        // However, the run methods require a SharedStore.
        // This indicates a slight mismatch: NodeTrait::exec doesn't get SharedStore.
        // The primary way to run a flow is via its run() or run_async() methods.
        // If a Flow is a node, its "execution" is its entire run.
        // The PostResult of the Flow's run becomes the "action" for the outer flow.
        // This suggests that the run/run_async methods of NodeTrait are more suitable here.
        // For now, let's make exec a no-op, as the main execution is handled by post.
        Ok(ExecResult::default())
    }

    fn post(
        &self,
        shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        // When a Flow is used as a node, its "result" is the PostResult of its internal run.
        self.run(shared_store) // Call the Flow's own run method
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store) // Default to sync version
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res) // Default to sync version
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        // When a Flow is used as a node, its "result" is the PostResult of its internal run.
        self.run_async(shared_store).await
    }

    // Successors for a Flow node would be transitions *from* this entire flow.
    // This requires Flow to store its own successors if it acts as a node.
    // This is not implemented yet, as Flow's primary role is to run an internal sequence.
    // To make Flow fully a NodeTrait that can be part of another Flow's transitions,
    // it would need its own successor map.
    // For now, these are no-ops.
    // fn add_successor(&mut self, _action: String, _node: Arc<dyn NodeTrait>) {}
    // fn get_successor(&self, _action: &str) -> Option<Arc<dyn NodeTrait>> { None }
}

// Removed placeholder Params struct
