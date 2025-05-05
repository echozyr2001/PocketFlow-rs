use anyhow::anyhow;
use std::{collections::HashMap, sync::Arc};

use super::{Action, ExecResult, PrepResult, Result, communication::SharedStore, node::BaseNode};

pub struct BatchProcessor {
    nodes: Vec<Arc<dyn BaseNode>>,
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
    pub fn add_node(&mut self, node: Arc<dyn BaseNode>) -> &mut Self {
        self.nodes.push(node);
        self
    }

    /// Process items in sequence (one after another)
    pub fn process_sequential(&self, shared_stores: Vec<SharedStore>) -> Vec<Result<Action>> {
        shared_stores
            .into_iter()
            .map(|store| self.process_single(&store))
            .collect()
    }

    /// Process a single item through all nodes
    pub fn process_single(&self, shared: &SharedStore) -> Result<Action> {
        if self.nodes.is_empty() {
            return Err(anyhow!("No nodes to process"));
        }

        // Run all nodes in sequence, storing intermediate results
        let mut results = Vec::new();
        for node in &self.nodes {
            results.push(node.run(shared)?);
        }

        // Return the action from the last node
        Ok(*results.last().unwrap())
    }

    /// Process items in parallel
    pub fn process_parallel(&self, shared_stores: Vec<SharedStore>) -> Vec<Result<Action>> {
        // // Only include rayon if BaseNode is Send + Sync
        // #[cfg(feature = "parallel")]
        // {
        //     use rayon::prelude::*;
        //     return shared_stores
        //         .into_par_iter()
        //         .map(|store| self.process_single(&store))
        //         .collect();
        // }

        // // Fallback to sequential processing if parallel feature is disabled
        // #[cfg(not(feature = "parallel"))]
        // {
        //     self.process_sequential(shared_stores)
        // }

        use rayon::prelude::*;
        shared_stores
            .into_par_iter()
            .map(|store| self.process_single(&store))
            .collect()
    }

    /// Group results by action
    pub fn group_results_by_action(
        &self,
        results: Vec<Result<Action>>,
    ) -> HashMap<String, Vec<usize>> {
        let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, result) in results.iter().enumerate() {
            if let Ok(action) = result {
                if let Some(act) = action.0 {
                    grouped.entry(act.to_string()).or_default().push(idx);
                } else {
                    grouped.entry("none".to_string()).or_default().push(idx);
                }
            } else {
                grouped.entry("error".to_string()).or_default().push(idx);
            }
        }

        grouped
    }
}

/// BatchNode wraps a BatchProcessor as a BaseNode
pub struct BatchNode {
    processor: BatchProcessor,
    stores: Vec<SharedStore>,
}

impl BatchNode {
    /// Create a new BatchNode
    pub fn new(processor: BatchProcessor, stores: Vec<SharedStore>) -> Self {
        Self { processor, stores }
    }

    /// Get the batch results
    pub fn get_results(&self) -> Vec<Result<Action>> {
        self.processor.process_sequential(self.stores.clone())
    }
}

impl BaseNode for BatchNode {
    fn prep(&self, _shared: &SharedStore) -> Result<PrepResult> {
        // Prepare batch data structure
        Ok(Default::default())
    }

    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        // Perform batch processing, but don't return the actual results here
        // Just indicate that processing was done
        let _results = self.processor.process_sequential(self.stores.clone());
        Ok(Default::default())
    }

    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        // Just signal completion
        Ok("batch_complete".into())
    }
}

/// BatchFlow combines batch processing with flow-based transitions
pub struct BatchFlow {
    processor: BatchProcessor,
    flows: Vec<Arc<dyn BaseNode>>,
    transition_map: HashMap<String, usize>,
}

impl BatchFlow {
    /// Create a new BatchFlow
    pub fn new(processor: BatchProcessor) -> Self {
        Self {
            processor,
            flows: Vec::new(),
            transition_map: HashMap::new(),
        }
    }

    /// Add a flow with associated action
    pub fn add_flow(&mut self, action: &str, flow: Arc<dyn BaseNode>) -> &mut Self {
        self.transition_map
            .insert(action.to_string(), self.flows.len());
        self.flows.push(flow);
        self
    }
    /// Process batch and route results to appropriate flows
    pub fn process(&self, stores: Vec<SharedStore>) -> HashMap<String, Vec<Result<Action>>> {
        let batch_results = self.processor.process_sequential(stores.clone());
        let grouped = self.processor.group_results_by_action(batch_results);

        let mut flow_results: HashMap<String, Vec<Result<Action>>> = HashMap::new();

        for (action, indices) in grouped {
            if let Some(&flow_idx) = self.transition_map.get(&action) {
                let flow = &self.flows[flow_idx];

                // Process each item with the matching flow
                for idx in indices {
                    if idx < stores.len() {
                        let result = flow.run(&stores[idx]);
                        flow_results.entry(action.clone()).or_default().push(result);
                    }
                }
            }
        }

        flow_results
    }
}
