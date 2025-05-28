//! 组合式节点实现
//! 
//! 使用组合模式而非继承构建节点

use crate::composition::behaviors::{PostBehavior, PrepBehavior, ExecBehavior};
use crate::core::{
    communication::SharedStore, node::NodeTrait, ExecResult, PostResult, PrepResult, Result,
};
use async_trait::async_trait;
use std::sync::Arc;

/// 组合式节点 - 通过组合行为组件而非继承实现功能
pub struct ComposableNode {
    /// 准备阶段的行为
    pub prep_behavior: Arc<dyn PrepBehavior>,
    /// 执行阶段的行为  
    pub exec_behavior: Arc<dyn ExecBehavior>,
    /// 后处理阶段的行为
    pub post_behavior: Arc<dyn PostBehavior>,
}

impl ComposableNode {
    /// 创建新的组合式节点
    pub fn new(
        prep_behavior: Arc<dyn PrepBehavior>,
        exec_behavior: Arc<dyn ExecBehavior>,
        post_behavior: Arc<dyn PostBehavior>,
    ) -> Self {
        Self {
            prep_behavior,
            exec_behavior,
            post_behavior,
        }
    }

    /// 获取准备行为的引用
    pub fn prep_behavior(&self) -> &dyn PrepBehavior {
        &*self.prep_behavior
    }

    /// 获取执行行为的引用
    pub fn exec_behavior(&self) -> &dyn ExecBehavior {
        &*self.exec_behavior
    }

    /// 获取后处理行为的引用
    pub fn post_behavior(&self) -> &dyn PostBehavior {
        &*self.post_behavior
    }

    /// 替换准备行为（消费原节点，返回新节点）
    pub fn with_prep_behavior(self, prep_behavior: Arc<dyn PrepBehavior>) -> Self {
        Self {
            prep_behavior,
            exec_behavior: self.exec_behavior,
            post_behavior: self.post_behavior,
        }
    }

    /// 替换执行行为（消费原节点，返回新节点）
    pub fn with_exec_behavior(self, exec_behavior: Arc<dyn ExecBehavior>) -> Self {
        Self {
            prep_behavior: self.prep_behavior,
            exec_behavior,
            post_behavior: self.post_behavior,
        }
    }

    /// 替换后处理行为（消费原节点，返回新节点）
    pub fn with_post_behavior(self, post_behavior: Arc<dyn PostBehavior>) -> Self {
        Self {
            prep_behavior: self.prep_behavior,
            exec_behavior: self.exec_behavior,
            post_behavior,
        }
    }
}

#[async_trait]
impl NodeTrait for ComposableNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep_behavior.prep(shared_store)
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec_behavior.exec(prep_res)
    }

    fn post(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post_behavior.post(shared_store, prep_res, exec_res)
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep_behavior.prep_async(shared_store).await
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.exec_behavior.exec_async(prep_res).await
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        self.post_behavior.post_async(shared_store, prep_res, exec_res).await
    }
}

// === 组合装饰器模式 ===

/// 重试装饰器 - 为任何 ExecBehavior 添加重试功能
pub struct RetryDecorator<T> {
    inner: T,
    max_retries: usize,
    wait_ms: u64,
}

impl<T> RetryDecorator<T> {
    pub fn new(inner: T, max_retries: usize, wait_ms: u64) -> Self {
        Self {
            inner,
            max_retries,
            wait_ms,
        }
    }
}

#[async_trait]
impl<T> ExecBehavior for RetryDecorator<T>
where
    T: ExecBehavior,
{
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.inner.exec(prep_result) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        std::thread::sleep(std::time::Duration::from_millis(self.wait_ms));
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    async fn exec_async(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.inner.exec_async(prep_result).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(self.wait_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
}

/// 缓存装饰器 - 为任何 ExecBehavior 添加缓存功能
pub struct CacheDecorator<T> {
    inner: T,
    cache: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, ExecResult>>>,
}

impl<T> CacheDecorator<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            cache: Default::default(),
        }
    }
    
    fn cache_key(&self, prep_result: &PrepResult) -> String {
        // 简单的缓存键生成策略 - 在实际应用中可以更复杂
        format!("{:?}", prep_result)
    }
}

#[async_trait]
impl<T> ExecBehavior for CacheDecorator<T>
where
    T: ExecBehavior,
{
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let key = self.cache_key(prep_result);
        
        // 尝试从缓存读取
        if let Some(cached) = self.cache.read().get(&key) {
            return Ok(cached.clone());
        }
        
        // 缓存未命中，执行原始逻辑
        let result = self.inner.exec(prep_result)?;
        
        // 写入缓存
        self.cache.write().insert(key, result.clone());
        
        Ok(result)
    }

    async fn exec_async(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let key = self.cache_key(prep_result);
        
        // 尝试从缓存读取
        if let Some(cached) = self.cache.read().get(&key) {
            return Ok(cached.clone());
        }
        
        // 缓存未命中，执行原始逻辑
        let result = self.inner.exec_async(prep_result).await?;
        
        // 写入缓存
        self.cache.write().insert(key, result.clone());
        
        Ok(result)
    }
}