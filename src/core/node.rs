use crate::core::{communication::SharedStore, r#type::Action};
use anyhow::Result;
use std::{thread::sleep, time::Duration};

/// The core Node trait that defines the basic interface for all nodes
pub trait Node {
    type PrepResult: Default + Clone;
    type ExecResult: Default + Clone;

    /// Prepare data from shared store
    fn prep(&self, _shared: &SharedStore) -> Result<Self::PrepResult> {
        Ok(Default::default())
    }

    /// Execute computation
    fn exec(&self, _prep_res: Self::PrepResult) -> Result<Self::ExecResult> {
        Ok(Default::default())
    }

    /// Process results and determine next action
    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: Self::PrepResult,
        _exec_res: Self::ExecResult,
    ) -> Result<Action> {
        Ok("default".into())
    }

    /// Run the node
    fn run(&self, shared: &SharedStore) -> Result<Action> {
        let prep_res = self.prep(shared)?;
        let exec_res = self.exec(prep_res.clone())?;
        self.post(shared, prep_res, exec_res)
    }
}

/// RetryableNode trait for nodes that support retry logic
pub trait RetryableNode: Node {
    fn get_max_retries(&self) -> u32;
    fn get_wait_ms(&self) -> u64;

    /// Retry-enabled run logic
    fn run_with_retry(&self, shared: &SharedStore) -> Result<Action> {
        let mut last_err = None;
        let max_retries = self.get_max_retries();
        let wait_ms = self.get_wait_ms();

        for attempt in 0..max_retries {
            let prep_res = self.prep(shared)?;
            match self.exec(prep_res.clone()) {
                Ok(exec_res) => {
                    return self.post(shared, prep_res, exec_res);
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt + 1 < max_retries {
                        sleep(Duration::from_millis(wait_ms));
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
    _phantom_p: std::marker::PhantomData<P>,
    _phantom_e: std::marker::PhantomData<E>,
}

impl<P, E> BaseNode<P, E>
where
    P: Default + Clone + 'static,
    E: Default + 'static,
{
    pub fn new(max_retries: u32, wait_ms: u64) -> Self {
        Self {
            max_retries,
            wait_ms,
            _phantom_p: std::marker::PhantomData,
            _phantom_e: std::marker::PhantomData,
        }
    }
}

impl<P, E> Node for BaseNode<P, E>
where
    // P: Default + Clone + 'static,
    // E: Default + Clone + 'static,
    P: Default + Clone,
    E: Default + Clone,
{
    type PrepResult = P;
    type ExecResult = E;
}

impl<P, E> RetryableNode for BaseNode<P, E>
where
    // P: Default + Clone + 'static,
    // E: Default + Clone + 'static,
    P: Default + Clone,
    E: Default + Clone,
{
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }

    fn get_wait_ms(&self) -> u64 {
        self.wait_ms
    }
}
