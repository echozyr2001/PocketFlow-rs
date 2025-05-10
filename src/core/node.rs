use crate::core::{
    communication::SharedStore,
    r#type::{ExecResult, PostResult, PrepResult},
};
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

// BaseNode and RetryableNode traits are removed.

// --- New NodeTrait ---
#[async_trait]
pub trait NodeTrait: Send + Sync {
    // Synchronous methods
    fn prep(&self, _shared_store: &dyn SharedStore) -> Result<PrepResult> {
        Ok(PrepResult::default())
    }

    fn exec(&self, _prep_res: &PrepResult) -> Result<ExecResult> {
        Ok(ExecResult::default())
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult, // prep_res often not needed in post if exec_res has all info
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::default())
    }

    // Asynchronous methods
    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(shared_store)
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post(shared_store, prep_res, exec_res)
    }

    // Successor management
    fn add_successor(&mut self, _action: String, _node: Arc<dyn NodeTrait>) {
        // Default: no-op. Nodes that manage successors should override.
    }

    fn get_successor(&self, _action: &str) -> Option<Arc<dyn NodeTrait>> {
        None
    }

    // Default run methods
    fn run_sync(&self, shared_store: &dyn SharedStore) -> Result<PostResult> {
        let prep_res = self.prep(shared_store)?;
        let exec_res = self.exec(&prep_res)?;
        self.post(shared_store, &prep_res, &exec_res)
    }

    async fn run_async(&self, shared_store: &dyn SharedStore) -> Result<PostResult> {
        let prep_res = self.prep_async(shared_store).await?;
        let exec_res = self.exec_async(&prep_res).await?;
        self.post_async(shared_store, &prep_res, &exec_res).await
    }
}

// --- RetryConfig ---
#[derive(Clone, Debug)]
pub struct RetryConfig {
    max_retries: usize,
    wait: Duration,
}

impl RetryConfig {
    pub fn new(max_retries: usize, wait_seconds: f64) -> Self {
        Self {
            max_retries,
            wait: Duration::from_secs_f64(wait_seconds),
        }
    }

    pub fn exec_with_retry<F>(&self, f: F) -> Result<ExecResult>
    where
        F: Fn() -> Result<ExecResult>,
    {
        if self.max_retries == 0 {
            return f();
        }
        for cur_retry in 0..self.max_retries {
            match f() {
                Ok(res) => return Ok(res),
                Err(e) => {
                    if cur_retry == self.max_retries - 1 {
                        return Err(e);
                    }
                    if !self.wait.is_zero() {
                        std::thread::sleep(self.wait);
                    }
                }
            }
        }
        bail!(
            "Exited retry loop unexpectedly (max_retries: {})",
            self.max_retries
        )
    }

    // Note: The original exec_async_with_retry used a generic Fut.
    // For simplicity with async_trait and common usage, often the closure itself is async.
    // However, the user's provided signature was:
    // F: Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<ExecResult>> + Send>>
    // This is more flexible if the future creation is complex. Let's stick to it.
    pub async fn exec_async_with_retry<F>(&self, f: F) -> Result<ExecResult>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<ExecResult>> + Send>>,
    {
        if self.max_retries == 0 {
            return f().await;
        }
        for cur_retry in 0..self.max_retries {
            match f().await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    if cur_retry == self.max_retries - 1 {
                        return Err(e);
                    }
                    if !self.wait.is_zero() {
                        tokio::time::sleep(self.wait).await; // Assuming tokio runtime
                    }
                }
            }
        }
        bail!(
            "Exited async retry loop unexpectedly (max_retries: {})",
            self.max_retries
        )
    }
}
