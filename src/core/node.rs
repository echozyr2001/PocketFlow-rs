use crate::core::communication::{Params, ParamsContainer, SharedStore};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// The core Node trait that defines the basic interface for all nodes
#[async_trait]
pub trait Node: Send + Sync + ParamsContainer {
    type PrepResult: Send + Default + Clone + 'static;
    type ExecResult: Send + Default + 'static;

    /// Prepare data from shared store
    async fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
        Ok(Default::default())
    }

    /// Execute computation
    async fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
        Ok(Default::default())
    }

    /// Process results and determine next action
    async fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: Self::PrepResult,
        _exec_res: Self::ExecResult,
    ) -> Result<Option<String>> {
        Ok(Some("default".to_string()))
    }

    /// Run the node (basic version without retries)
    async fn run(&self, shared: &SharedStore) -> Result<Option<String>> {
        let prep_res = self.prep(shared).await?;
        let exec_res = self.exec(prep_res.clone()).await?;
        self.post(shared, prep_res, exec_res).await
    }
}

/// RetryableNode trait for nodes that support retry logic
#[async_trait]
pub trait RetryableNode: Node {
    fn get_max_retries(&self) -> u32;
    fn get_wait_ms(&self) -> u64;

    /// Retry-enabled run logic
    async fn run_with_retry(&self, shared: &SharedStore) -> Result<Option<String>> {
        let mut last_err = None;
        let max_retries = self.get_max_retries();
        let wait_ms = self.get_wait_ms();

        for attempt in 0..max_retries {
            let prep_res = self.prep(shared).await?;
            match self.exec(prep_res.clone()).await {
                Ok(exec_res) => {
                    return self.post(shared, prep_res, exec_res).await;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt + 1 < max_retries {
                        sleep(Duration::from_millis(wait_ms)).await;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Node execution failed")))
    }
}

/// Base implementation for nodes with retry capability and params
pub struct BaseNode<P, E> {
    max_retries: u32,
    wait_ms: u64,
    params: Params,
    successors: HashMap<String, Box<dyn Node<PrepResult = P, ExecResult = E>>>,
}

impl<P, E> BaseNode<P, E>
where
    P: Send + Default + Clone + 'static,
    E: Send + Default + 'static,
{
    pub fn new(max_retries: u32, wait_ms: u64) -> Self {
        Self {
            max_retries,
            wait_ms,
            params: Params::new(),
            successors: HashMap::new(),
        }
    }

    pub fn add_successor(
        &mut self,
        action: &str,
        node: Box<dyn Node<PrepResult = P, ExecResult = E>>,
    ) {
        self.successors.insert(action.to_string(), node);
    }

    pub fn successors(&self) -> &HashMap<String, Box<dyn Node<PrepResult = P, ExecResult = E>>> {
        &self.successors
    }
}

impl<P, E> ParamsContainer for BaseNode<P, E> {
    fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    fn get_params(&self) -> &Params {
        &self.params
    }

    fn get_params_mut(&mut self) -> &mut Params {
        &mut self.params
    }
}

#[async_trait]
impl<P, E> Node for BaseNode<P, E>
where
    P: Send + Default + Clone + 'static,
    E: Send + Default + 'static,
{
    type PrepResult = P;
    type ExecResult = E;
}

impl<P, E> RetryableNode for BaseNode<P, E>
where
    P: Send + Default + Clone + 'static,
    E: Send + Default + 'static,
{
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }

    fn get_wait_ms(&self) -> u64 {
        self.wait_ms
    }
}
