//! 行为组件实现集合
//! 
//! 提供常用的行为组件实现

pub mod llm;
pub mod data;
pub mod logging;

use crate::core::{communication::SharedStore, ExecResult, PostResult, PrepResult, Result};
use async_trait::async_trait;
use std::sync::Arc;

pub use llm::*;
pub use data::*; 
pub use logging::*;

/// 准备阶段行为组件
#[async_trait]
pub trait PrepBehavior: Send + Sync {
    /// 同步准备操作
    fn prep(&self, store: &dyn SharedStore) -> Result<PrepResult>;
    
    /// 异步准备操作
    async fn prep_async(&self, store: &dyn SharedStore) -> Result<PrepResult> {
        self.prep(store)
    }
}

/// 执行阶段行为组件  
#[async_trait]
pub trait ExecBehavior: Send + Sync {
    /// 同步执行操作
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult>;
    
    /// 异步执行操作
    async fn exec_async(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        self.exec(prep_result)
    }
}

/// 后处理阶段行为组件
#[async_trait]
pub trait PostBehavior: Send + Sync {
    /// 同步后处理操作
    fn post(
        &self,
        store: &dyn SharedStore,
        prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<PostResult>;
    
    /// 异步后处理操作
    async fn post_async(
        &self,
        store: &dyn SharedStore,
        prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<PostResult> {
        self.post(store, prep_result, exec_result)
    }
}

// === 基础行为实现 ===

/// 默认准备行为 - 不执行任何操作
#[derive(Default, Clone)]
pub struct DefaultPrepBehavior;

#[async_trait]
impl PrepBehavior for DefaultPrepBehavior {
    fn prep(&self, _store: &dyn SharedStore) -> Result<PrepResult> {
        Ok(PrepResult::default())
    }
}

/// 默认执行行为 - 返回默认结果
#[derive(Default, Clone)]
pub struct DefaultExecBehavior;

#[async_trait]
impl ExecBehavior for DefaultExecBehavior {
    fn exec(&self, _prep_result: &PrepResult) -> Result<ExecResult> {
        Ok(ExecResult::default())
    }
}

/// 默认后处理行为 - 返回默认动作
#[derive(Default, Clone)]
pub struct DefaultPostBehavior;

#[async_trait]
impl PostBehavior for DefaultPostBehavior {
    fn post(
        &self,
        _store: &dyn SharedStore,
        _prep_result: &PrepResult,
        _exec_result: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::default())
    }
}

// === 函数式行为包装器 ===

/// 将函数包装为 PrepBehavior
pub struct FnPrepBehavior<F> {
    pub func: F,
}

impl<F> FnPrepBehavior<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> PrepBehavior for FnPrepBehavior<F>
where
    F: Fn(&dyn SharedStore) -> Result<PrepResult> + Send + Sync,
{
    fn prep(&self, store: &dyn SharedStore) -> Result<PrepResult> {
        (self.func)(store)
    }
}

/// 将函数包装为 ExecBehavior
pub struct FnExecBehavior<F> {
    pub func: F,
}

impl<F> FnExecBehavior<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> ExecBehavior for FnExecBehavior<F>
where
    F: Fn(&PrepResult) -> Result<ExecResult> + Send + Sync,
{
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        (self.func)(prep_result)
    }
}

/// 将函数包装为 PostBehavior
pub struct FnPostBehavior<F> {
    pub func: F,
}

impl<F> FnPostBehavior<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F> PostBehavior for FnPostBehavior<F>
where
    F: Fn(&dyn SharedStore, &PrepResult, &ExecResult) -> Result<PostResult> + Send + Sync,
{
    fn post(
        &self,
        store: &dyn SharedStore,
        prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<PostResult> {
        (self.func)(store, prep_result, exec_result)
    }
}

// === 类型转换辅助工具 ===

/// 将任何实现了 PrepBehavior 的类型转换为 Arc<dyn PrepBehavior>
pub fn prep_behavior<T: PrepBehavior + 'static>(behavior: T) -> Arc<dyn PrepBehavior> {
    Arc::new(behavior)
}

/// 将任何实现了 ExecBehavior 的类型转换为 Arc<dyn ExecBehavior>  
pub fn exec_behavior<T: ExecBehavior + 'static>(behavior: T) -> Arc<dyn ExecBehavior> {
    Arc::new(behavior)
}

/// 将任何实现了 PostBehavior 的类型转换为 Arc<dyn PostBehavior>
pub fn post_behavior<T: PostBehavior + 'static>(behavior: T) -> Arc<dyn PostBehavior> {
    Arc::new(behavior)
}

// === 便利宏 ===

/// 创建函数式 PrepBehavior 的便利宏
#[macro_export]
macro_rules! prep_fn {
    ($func:expr) => {
        $crate::composition::prep_behavior($crate::composition::FnPrepBehavior::new($func))
    };
}

/// 创建函数式 ExecBehavior 的便利宏
#[macro_export]
macro_rules! exec_fn {
    ($func:expr) => {
        $crate::composition::exec_behavior($crate::composition::FnExecBehavior::new($func))
    };
}

/// 创建函数式 PostBehavior 的便利宏
#[macro_export]
macro_rules! post_fn {
    ($func:expr) => {
        $crate::composition::post_behavior($crate::composition::FnPostBehavior::new($func))
    };
}