//! 日志相关行为组件

use crate::composition::behaviors::{PostBehavior, PrepBehavior};
use crate::core::{communication::SharedStore, PostResult, PrepResult, Result};
use async_trait::async_trait;
use serde_json::json;

/// 日志准备行为 - 记录节点开始执行
#[derive(Clone)]
pub struct LogStartPrepBehavior {
    pub node_name: String,
    pub log_level: LogLevel,
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogStartPrepBehavior {
    pub fn new(node_name: impl Into<String>) -> Self {
        Self {
            node_name: node_name.into(),
            log_level: LogLevel::Info,
        }
    }
    
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }
}

#[async_trait]
impl PrepBehavior for LogStartPrepBehavior {
    fn prep(&self, store: &dyn SharedStore) -> Result<PrepResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let message = format!(
            "[{}] Starting node: {} at {}",
            format!("{:?}", self.log_level).to_uppercase(),
            self.node_name,
            timestamp
        );
        
        match self.log_level {
            LogLevel::Debug => tracing::debug!("{}", message),
            LogLevel::Info => tracing::info!("{}", message),
            LogLevel::Warn => tracing::warn!("{}", message),
            LogLevel::Error => tracing::error!("{}", message),
        }
        
        println!("{}", message);
        
        // 可选择将日志信息存储到存储中
        store.insert_value("last_log", std::sync::Arc::new(message.clone()));
        
        Ok(PrepResult::new(json!({
            "node_name": self.node_name,
            "start_time": timestamp,
            "log_message": message
        })))
    }
}

/// 性能监控后处理行为
#[derive(Clone)]
pub struct PerformanceLogPostBehavior {
    pub node_name: String,
}

impl PerformanceLogPostBehavior {
    pub fn new(node_name: impl Into<String>) -> Self {
        Self {
            node_name: node_name.into(),
        }
    }
}

#[async_trait]
impl PostBehavior for PerformanceLogPostBehavior {
    fn post(
        &self,
        store: &dyn SharedStore,
        prep_result: &PrepResult,
        _exec_result: &crate::core::ExecResult,
    ) -> Result<PostResult> {
        let end_time = chrono::Utc::now().to_rfc3339();
        
        if let Some(start_time_val) = prep_result.get_value("start_time") {
            if let Some(start_time_str) = start_time_val.as_str() {
                if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start_time_str) {
                    let duration = chrono::Utc::now().signed_duration_since(start_time);
                    let duration_ms = duration.num_milliseconds();
                    
                    let perf_message = format!(
                        "[PERF] Node '{}' completed in {}ms",
                        self.node_name, duration_ms
                    );
                    
                    tracing::info!("{}", perf_message);
                    println!("{}", perf_message);
                    
                    // 存储性能数据
                    store.insert_value("performance_data", std::sync::Arc::new(json!({
                        "node_name": self.node_name,
                        "start_time": start_time_str,
                        "end_time": end_time,
                        "duration_ms": duration_ms
                    })));
                }
            }
        }
        
        Ok(PostResult::default())
    }
}