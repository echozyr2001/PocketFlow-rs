use crate::core::{
    communication::SharedStore,
    r#type::{Action, ExecResult, PrepResult},
};
use anyhow::Result;
use std::{thread::sleep, time::Duration};

/// The core Node trait that defines the basic interface for all nodes
// #[cfg(feature = "parallel")]
pub trait BaseNode: Send + Sync {
    /// Prepare data from shared store
    fn prep(&self, _shared: &SharedStore) -> Result<PrepResult> {
        Ok(Default::default())
    }

    /// Execute computation
    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        Ok(Default::default())
    }

    /// Process results and determine next action
    fn post(
        &self,
        _shared: &SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<Action> {
        Ok(Action::default())
    }

    /// Run the node
    fn run(&self, shared: &SharedStore) -> Result<Action> {
        let prep_res = self.prep(shared)?;
        let exec_res = self.exec(&prep_res)?;
        self.post(shared, &prep_res, &exec_res)
    }
}

// #[cfg(not(feature = "parallel"))]
// pub trait BaseNode {
// ...
// }

/// RetryableNode trait for nodes that support retry logic
pub trait RetryableNode: BaseNode {
    fn get_max_retries(&self) -> u32;
    fn get_wait_ms(&self) -> u64;

    /// Retry-enabled run logic
    fn run_with_retry(&self, shared: &SharedStore) -> Result<Action> {
        let mut last_err = None;
        let max_retries = self.get_max_retries();
        let wait_ms = self.get_wait_ms();

        for attempt in 0..max_retries {
            let prep_res = self.prep(shared)?;
            match self.exec(&prep_res) {
                Ok(exec_res) => {
                    return self.post(shared, &prep_res, &exec_res);
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
