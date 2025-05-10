use anyhow::{Result, anyhow};
use std::{collections::HashMap, sync::Arc};

use super::{
    communication::{BaseSharedStore, SharedStore},
    node::NodeTrait,
    r#type::{ExecResult, PostResult, PrepResult},
};

pub struct BatchProcessor {
    nodes: Vec<Arc<dyn NodeTrait>>,
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchProcessor {
    /// Create a new BatchProcessor
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a node to the batch processor
    pub fn add_node(&mut self, node: Arc<dyn NodeTrait>) -> &mut Self {
        self.nodes.push(node);
        self
    }

    /// Process items in sequence (one after another)
    pub fn process_sequential(
        &self,
        shared_stores: Vec<BaseSharedStore>,
    ) -> Vec<Result<PostResult>> {
        shared_stores
            .into_iter()
            .map(|store| self.process_single(&store))
            .collect()
    }

    /// Process a single item through all nodes
    pub fn process_single(&self, shared: &BaseSharedStore) -> Result<PostResult> {
        if self.nodes.is_empty() {
            return Err(anyhow!("No nodes to process"));
        }

        let mut results = Vec::new();
        for node in &self.nodes {
            results.push(node.run(shared)?);
        }

        results
            .last()
            .cloned()
            .ok_or_else(|| anyhow!("No results from node processing"))
    }

    /// Process items in parallel
    pub fn process_parallel(&self, shared_stores: Vec<BaseSharedStore>) -> Vec<Result<PostResult>> {
        use rayon::prelude::*;
        shared_stores
            .into_par_iter()
            .map(|store| self.process_single(&store))
            .collect()
    }

    /// Group results by action string from PostResult
    pub fn group_results_by_action(
        &self,
        results: &[Result<PostResult>],
    ) -> HashMap<String, Vec<usize>> {
        let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, result) in results.iter().enumerate() {
            if let Ok(post_result) = result {
                let action_str = post_result.as_str();
                if action_str.is_empty() {
                    grouped
                        .entry("default_empty_post_result".to_string())
                        .or_default()
                        .push(idx);
                } else {
                    grouped.entry(action_str.to_string()).or_default().push(idx);
                }
            } else {
                grouped.entry("error".to_string()).or_default().push(idx);
            }
        }
        grouped
    }
}

/// BatchNode wraps a BatchProcessor as a NodeTrait
pub struct BatchNode {
    processor: BatchProcessor,
    stores: Vec<BaseSharedStore>,
}

impl BatchNode {
    /// Create a new BatchNode
    pub fn new(processor: BatchProcessor, stores: Vec<BaseSharedStore>) -> Self {
        Self { processor, stores }
    }

    /// Get the batch results
    pub fn get_results(&self) -> Vec<Result<PostResult>> {
        self.processor.process_sequential(self.stores.clone())
    }
}

#[async_trait::async_trait]
impl NodeTrait for BatchNode {
    fn prep(&self, _shared_store: &dyn SharedStore) -> Result<PrepResult> {
        Ok(Default::default())
    }

    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        let _results = self.processor.process_sequential(self.stores.clone());
        Ok(Default::default())
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::from("batch_complete"))
    }
}

/// BatchFlow combines batch processing with flow-based transitions
pub struct BatchFlow {
    processor: BatchProcessor,
    flow_transitions: HashMap<String, Arc<dyn NodeTrait>>,
}

impl BatchFlow {
    pub fn new(processor: BatchProcessor) -> Self {
        Self {
            processor,
            flow_transitions: HashMap::new(),
        }
    }

    pub fn add_flow(&mut self, action_key: &str, flow_node: Arc<dyn NodeTrait>) -> &mut Self {
        self.flow_transitions
            .insert(action_key.to_string(), flow_node);
        self
    }

    pub fn process(
        &self,
        stores: Vec<BaseSharedStore>,
    ) -> HashMap<String, Vec<Result<PostResult>>> {
        let batch_post_results = self.processor.process_sequential(stores.clone());

        let mut grouped_indices_by_action: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, res) in batch_post_results.iter().enumerate() {
            if let Ok(post_result) = res {
                grouped_indices_by_action
                    .entry(post_result.as_str().to_string())
                    .or_default()
                    .push(idx);
            } else {
                // Errors from batch_processor are not passed to sub-flows for now.
            }
        }

        let mut final_flow_results: HashMap<String, Vec<Result<PostResult>>> = HashMap::new();

        for (action_key, indices) in grouped_indices_by_action {
            if let Some(flow_to_run) = self.flow_transitions.get(&action_key) {
                let mut results_for_this_flow: Vec<Result<PostResult>> = Vec::new();
                for &original_store_idx in &indices {
                    if original_store_idx < stores.len() {
                        let item_store = &stores[original_store_idx];
                        results_for_this_flow.push(flow_to_run.run(item_store));
                    }
                }
                final_flow_results.insert(action_key, results_for_this_flow);
            }
        }
        final_flow_results
    }
}
