//! 组合式构建器
//! 
//! 提供类型安全、流式的组合节点构建接口

use crate::composition::behaviors::{
    DefaultExecBehavior, DefaultPostBehavior, DefaultPrepBehavior, ExecBehavior, PostBehavior,
    PrepBehavior,
};
use crate::composition::node::{CacheDecorator, ComposableNode, RetryDecorator};
use crate::core::{ExecResult, PostResult, PrepResult, Result};
use std::sync::Arc;

/// 节点构建器 - 提供流式 API 构建组合节点
pub struct NodeBuilder {
    prep_behavior: Option<Arc<dyn PrepBehavior>>,
    exec_behavior: Option<Arc<dyn ExecBehavior>>,
    post_behavior: Option<Arc<dyn PostBehavior>>,
}

impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            prep_behavior: None,
            exec_behavior: None,
            post_behavior: None,
        }
    }

    /// 设置准备行为
    pub fn with_prep<T: PrepBehavior + 'static>(mut self, behavior: T) -> Self {
        self.prep_behavior = Some(Arc::new(behavior));
        self
    }

    /// 设置准备行为（Arc 版本）
    pub fn with_prep_arc(mut self, behavior: Arc<dyn PrepBehavior>) -> Self {
        self.prep_behavior = Some(behavior);
        self
    }

    /// 设置执行行为
    pub fn with_exec<T: ExecBehavior + 'static>(mut self, behavior: T) -> Self {
        self.exec_behavior = Some(Arc::new(behavior));
        self
    }

    /// 设置执行行为（Arc 版本）
    pub fn with_exec_arc(mut self, behavior: Arc<dyn ExecBehavior>) -> Self {
        self.exec_behavior = Some(behavior);
        self
    }

    /// 设置后处理行为
    pub fn with_post<T: PostBehavior + 'static>(mut self, behavior: T) -> Self {
        self.post_behavior = Some(Arc::new(behavior));
        self
    }

    /// 设置后处理行为（Arc 版本）
    pub fn with_post_arc(mut self, behavior: Arc<dyn PostBehavior>) -> Self {
        self.post_behavior = Some(behavior);
        self
    }

    /// 构建节点，使用默认行为填充缺失的组件
    pub fn build(self) -> ComposableNode {
        ComposableNode::new(
            self.prep_behavior
                .unwrap_or_else(|| Arc::new(DefaultPrepBehavior)),
            self.exec_behavior
                .unwrap_or_else(|| Arc::new(DefaultExecBehavior)),
            self.post_behavior
                .unwrap_or_else(|| Arc::new(DefaultPostBehavior)),
        )
    }

    /// 构建节点，如果有组件缺失则返回错误
    pub fn build_strict(self) -> Result<ComposableNode> {
        Ok(ComposableNode::new(
            self.prep_behavior
                .ok_or_else(|| anyhow::anyhow!("PrepBehavior is required"))?,
            self.exec_behavior
                .ok_or_else(|| anyhow::anyhow!("ExecBehavior is required"))?,
            self.post_behavior
                .ok_or_else(|| anyhow::anyhow!("PostBehavior is required"))?,
        ))
    }
}

/// 装饰器构建器 - 为现有行为添加装饰器
pub struct DecoratorBuilder<T> {
    inner: T,
}

impl<T> DecoratorBuilder<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// 添加重试装饰器
    pub fn with_retry(self, max_retries: usize, wait_ms: u64) -> DecoratorBuilder<RetryDecorator<T>> {
        DecoratorBuilder::new(RetryDecorator::new(self.inner, max_retries, wait_ms))
    }

    /// 添加缓存装饰器  
    pub fn with_cache(self) -> DecoratorBuilder<CacheDecorator<T>> {
        DecoratorBuilder::new(CacheDecorator::new(self.inner))
    }

    /// 完成装饰器链构建
    pub fn build(self) -> T {
        self.inner
    }
}

/// 为任意类型提供装饰器构建功能
pub trait Decoratable<T> {
    fn decorators(self) -> DecoratorBuilder<T>;
}

impl<T> Decoratable<T> for T {
    fn decorators(self) -> DecoratorBuilder<T> {
        DecoratorBuilder::new(self)
    }
}

// === 便利构建函数 ===

/// 快速创建简单节点的便利函数
pub fn simple_node() -> NodeBuilder {
    NodeBuilder::new()
}

/// 快速创建空节点（使用默认行为）
pub fn empty_node() -> ComposableNode {
    NodeBuilder::new().build()
}

/// 使用函数式 API 构建节点
/// 
/// # Example
/// ```
/// use pocketflow_rs::composition::*;
/// use pocketflow_rs::core::{PrepResult, ExecResult, PostResult};
/// 
/// let node = node_from_fns(
///     |_store| Ok(PrepResult::default()),
///     |_prep| Ok(ExecResult::default()),
///     |_store, _prep, _exec| Ok(PostResult::default()),
/// );
/// ```
pub fn node_from_fns<P, E, T>(
    prep_fn: P,
    exec_fn: E,
    post_fn: T,
) -> ComposableNode
where
    P: Fn(&dyn crate::core::communication::SharedStore) -> Result<PrepResult> + Send + Sync + 'static,
    E: Fn(&PrepResult) -> Result<ExecResult> + Send + Sync + 'static,
    T: Fn(&dyn crate::core::communication::SharedStore, &PrepResult, &ExecResult) -> Result<PostResult> + Send + Sync + 'static,
{
    NodeBuilder::new()
        .with_prep(crate::composition::behaviors::FnPrepBehavior::new(prep_fn))
        .with_exec(crate::composition::behaviors::FnExecBehavior::new(exec_fn))
        .with_post(crate::composition::behaviors::FnPostBehavior::new(post_fn))
        .build()
}

// === 便利宏 ===

/// 用于快速构建节点的宏
/// 
/// # Example
/// ```
/// use pocketflow_rs::compose_node;
/// use pocketflow_rs::core::{PrepResult, ExecResult, PostResult, communication::SharedStore};
/// 
/// let node = compose_node! {
///     exec: |_prep: &PrepResult| Ok(ExecResult::default()),
/// };
/// ```
#[macro_export]
macro_rules! compose_node {
    (
        prep: $prep:expr,
        exec: $exec:expr,
        post: $post:expr $(,)?
    ) => {
        $crate::composition::NodeBuilder::new()
            .with_prep($crate::composition::behaviors::FnPrepBehavior::new($prep))
            .with_exec($crate::composition::behaviors::FnExecBehavior::new($exec))
            .with_post($crate::composition::behaviors::FnPostBehavior::new($post))
            .build()
    };
    
    (
        prep: $prep:expr,
        exec: $exec:expr $(,)?
    ) => {
        $crate::composition::NodeBuilder::new()
            .with_prep($crate::composition::behaviors::FnPrepBehavior::new($prep))
            .with_exec($crate::composition::behaviors::FnExecBehavior::new($exec))
            .build()
    };
    
    (
        exec: $exec:expr $(,)?
    ) => {
        $crate::composition::NodeBuilder::new()
            .with_exec($crate::composition::behaviors::FnExecBehavior::new($exec))
            .build()
    };
}