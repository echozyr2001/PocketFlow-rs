//! 监控工具模块
//! 
//! 提供性能监控、执行跟踪和资源监控功能

use crate::core::{Store, NodeId, PostResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, debug, warn};

/// 监控工具特征
pub trait MonitorTool: Send + Sync {
    /// 开始监控节点执行
    fn start_monitoring(&mut self, node_id: &NodeId);
    
    /// 停止监控节点执行
    fn stop_monitoring(&mut self, node_id: &NodeId);
    
    /// 记录性能指标
    fn record_metric(&mut self, name: &str, value: f64);
    
    /// 获取性能报告
    fn get_performance_report(&self) -> PerformanceReport;
    
    /// 获取资源使用情况
    fn get_resource_usage(&self) -> ResourceUsage;
    
    /// 重置监控数据
    fn reset(&mut self);
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub total_execution_time: Duration,
    pub node_metrics: HashMap<NodeId, NodeMetrics>,
    pub custom_metrics: HashMap<String, f64>,
    pub throughput: f64,
}

/// 节点性能指标
#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub execution_time: Duration,
    pub memory_usage: u64,
    pub execution_count: u64,
    pub average_time: Duration,
}

/// 资源使用情况
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used: u64,
    pub cpu_usage: f64,
    pub disk_io: u64,
    pub network_io: u64,
}

/// 性能监控器
/// 
/// 监控流程执行的性能指标
#[derive(Debug)]
pub struct PerformanceMonitor {
    node_start_times: HashMap<NodeId, Instant>,
    node_metrics: HashMap<NodeId, NodeMetrics>,
    custom_metrics: HashMap<String, f64>,
    total_start_time: Option<Instant>,
    monitoring_active: bool,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            node_start_times: HashMap::new(),
            node_metrics: HashMap::new(),
            custom_metrics: HashMap::new(),
            total_start_time: None,
            monitoring_active: false,
        }
    }
    
    /// 开始监控会话
    pub fn start_session(&mut self) {
        self.total_start_time = Some(Instant::now());
        self.monitoring_active = true;
        info!("性能监控器: 开始监控会话");
    }
    
    /// 结束监控会话
    pub fn end_session(&mut self) {
        self.monitoring_active = false;
        info!("性能监控器: 结束监控会话");
    }
    
    /// 记录节点执行完成
    pub fn on_node_complete(&mut self, node_id: &NodeId, result: &PostResult) {
        if let Some(start_time) = self.node_start_times.remove(node_id) {
            let execution_time = start_time.elapsed();
            
            let metrics = self.node_metrics.entry(node_id.clone()).or_insert(NodeMetrics {
                execution_time: Duration::ZERO,
                memory_usage: 0,
                execution_count: 0,
                average_time: Duration::ZERO,
            });
            
            metrics.execution_count += 1;
            metrics.execution_time += execution_time;
            metrics.average_time = metrics.execution_time / metrics.execution_count as u32;
            
            debug!("性能监控器: 节点 {:?} 执行时间: {:?}", node_id, execution_time);
        }
    }
}

impl MonitorTool for PerformanceMonitor {
    fn start_monitoring(&mut self, node_id: &NodeId) {
        if self.monitoring_active {
            self.node_start_times.insert(node_id.clone(), Instant::now());
            debug!("性能监控器: 开始监控节点 {:?}", node_id);
        }
    }
    
    fn stop_monitoring(&mut self, node_id: &NodeId) {
        if let Some(start_time) = self.node_start_times.remove(node_id) {
            let execution_time = start_time.elapsed();
            debug!("性能监控器: 停止监控节点 {:?}，执行时间: {:?}", node_id, execution_time);
        }
    }
    
    fn record_metric(&mut self, name: &str, value: f64) {
        self.custom_metrics.insert(name.to_string(), value);
        debug!("性能监控器: 记录指标 {}: {}", name, value);
    }
    
    fn get_performance_report(&self) -> PerformanceReport {
        let total_execution_time = self.total_start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);
            
        PerformanceReport {
            total_execution_time,
            node_metrics: self.node_metrics.clone(),
            custom_metrics: self.custom_metrics.clone(),
            throughput: self.calculate_throughput(),
        }
    }
    
    fn get_resource_usage(&self) -> ResourceUsage {
        // 简化实现，实际中可以集成系统监控
        ResourceUsage {
            memory_used: 0,
            cpu_usage: 0.0,
            disk_io: 0,
            network_io: 0,
        }
    }
    
    fn reset(&mut self) {
        self.node_start_times.clear();
        self.node_metrics.clear();
        self.custom_metrics.clear();
        self.total_start_time = None;
        info!("性能监控器: 重置监控数据");
    }
}

impl PerformanceMonitor {
    fn calculate_throughput(&self) -> f64 {
        let total_nodes = self.node_metrics.len() as f64;
        let total_time = self.total_start_time
            .map(|start| start.elapsed().as_secs_f64())
            .unwrap_or(1.0);
        
        total_nodes / total_time
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行跟踪器
/// 
/// 跟踪流程执行路径和状态变化
#[derive(Debug)]
pub struct ExecutionTracer {
    execution_trace: Vec<TraceEvent>,
    max_events: usize,
    tracing_active: bool,
}

/// 跟踪事件
#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub timestamp: Instant,
    pub node_id: NodeId,
    pub event_type: TraceEventType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum TraceEventType {
    NodeStart,
    NodeComplete,
    StateChange,
    Error,
}

impl ExecutionTracer {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }
    
    pub fn with_capacity(max_events: usize) -> Self {
        Self {
            execution_trace: Vec::new(),
            max_events,
            tracing_active: false,
        }
    }
    
    /// 启用跟踪
    pub fn enable_tracing(&mut self) {
        self.tracing_active = true;
        info!("执行跟踪器: 启用跟踪");
    }
    
    /// 禁用跟踪
    pub fn disable_tracing(&mut self) {
        self.tracing_active = false;
        info!("执行跟踪器: 禁用跟踪");
    }
    
    /// 记录事件
    pub fn record_event(&mut self, node_id: NodeId, event_type: TraceEventType, details: String) {
        if !self.tracing_active {
            return;
        }
        
        if self.execution_trace.len() >= self.max_events {
            self.execution_trace.remove(0);
        }
        
        self.execution_trace.push(TraceEvent {
            timestamp: Instant::now(),
            node_id,
            event_type,
            details,
        });
    }
    
    /// 获取执行跟踪
    pub fn get_trace(&self) -> &[TraceEvent] {
        &self.execution_trace
    }
    
    /// 生成跟踪报告
    pub fn generate_trace_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== 执行跟踪报告 ===\n");
        
        for event in &self.execution_trace {
            report.push_str(&format!(
                "[{:?}] {:?} - {:?}: {}\n",
                event.timestamp, event.node_id, event.event_type, event.details
            ));
        }
        
        report
    }
}

impl MonitorTool for ExecutionTracer {
    fn start_monitoring(&mut self, node_id: &NodeId) {
        self.record_event(node_id.clone(), TraceEventType::NodeStart, "节点开始执行".to_string());
    }
    
    fn stop_monitoring(&mut self, node_id: &NodeId) {
        self.record_event(node_id.clone(), TraceEventType::NodeComplete, "节点执行完成".to_string());
    }
    
    fn record_metric(&mut self, name: &str, value: f64) {
        // 执行跟踪器主要记录事件，不直接记录指标
        debug!("执行跟踪器: 指标 {}: {}", name, value);
    }
    
    fn get_performance_report(&self) -> PerformanceReport {
        // 执行跟踪器不提供详细的性能报告
        PerformanceReport {
            total_execution_time: Duration::ZERO,
            node_metrics: HashMap::new(),
            custom_metrics: HashMap::new(),
            throughput: 0.0,
        }
    }
    
    fn get_resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            memory_used: 0,
            cpu_usage: 0.0,
            disk_io: 0,
            network_io: 0,
        }
    }
    
    fn reset(&mut self) {
        self.execution_trace.clear();
        info!("执行跟踪器: 重置跟踪数据");
    }
}

impl Default for ExecutionTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源监控器
/// 
/// 监控系统资源使用情况
#[derive(Debug)]
pub struct ResourceMonitor {
    resource_samples: Vec<ResourceSample>,
    monitoring_interval: Duration,
    max_samples: usize,
    monitoring_active: bool,
}

/// 资源样本
#[derive(Debug, Clone)]
pub struct ResourceSample {
    pub timestamp: Instant,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub disk_io: u64,
    pub network_io: u64,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            resource_samples: Vec::new(),
            monitoring_interval: Duration::from_secs(1),
            max_samples: 3600, // 1小时的样本
            monitoring_active: false,
        }
    }
    
    /// 设置监控间隔
    pub fn set_monitoring_interval(&mut self, interval: Duration) {
        self.monitoring_interval = interval;
    }
    
    /// 开始资源监控
    pub fn start_resource_monitoring(&mut self) {
        self.monitoring_active = true;
        info!("资源监控器: 开始资源监控");
    }
    
    /// 停止资源监控
    pub fn stop_resource_monitoring(&mut self) {
        self.monitoring_active = false;
        info!("资源监控器: 停止资源监控");
    }
    
    /// 采样资源使用情况
    pub fn sample_resources(&mut self) {
        if !self.monitoring_active {
            return;
        }
        
        if self.resource_samples.len() >= self.max_samples {
            self.resource_samples.remove(0);
        }
        
        // 简化实现，实际中需要调用系统API
        let sample = ResourceSample {
            timestamp: Instant::now(),
            memory_usage: 0,
            cpu_usage: 0.0,
            disk_io: 0,
            network_io: 0,
        };
        
        self.resource_samples.push(sample);
    }
    
    /// 获取资源历史
    pub fn get_resource_history(&self) -> &[ResourceSample] {
        &self.resource_samples
    }
    
    /// 计算平均资源使用
    pub fn get_average_usage(&self) -> ResourceUsage {
        if self.resource_samples.is_empty() {
            return ResourceUsage {
                memory_used: 0,
                cpu_usage: 0.0,
                disk_io: 0,
                network_io: 0,
            };
        }
        
        let count = self.resource_samples.len() as f64;
        let sum_memory: u64 = self.resource_samples.iter().map(|s| s.memory_usage).sum();
        let sum_cpu: f64 = self.resource_samples.iter().map(|s| s.cpu_usage).sum();
        let sum_disk: u64 = self.resource_samples.iter().map(|s| s.disk_io).sum();
        let sum_network: u64 = self.resource_samples.iter().map(|s| s.network_io).sum();
        
        ResourceUsage {
            memory_used: (sum_memory as f64 / count) as u64,
            cpu_usage: sum_cpu / count,
            disk_io: (sum_disk as f64 / count) as u64,
            network_io: (sum_network as f64 / count) as u64,
        }
    }
}

impl MonitorTool for ResourceMonitor {
    fn start_monitoring(&mut self, _node_id: &NodeId) {
        self.sample_resources();
    }
    
    fn stop_monitoring(&mut self, _node_id: &NodeId) {
        self.sample_resources();
    }
    
    fn record_metric(&mut self, name: &str, value: f64) {
        debug!("资源监控器: 记录指标 {}: {}", name, value);
    }
    
    fn get_performance_report(&self) -> PerformanceReport {
        // 资源监控器主要关注资源，不提供详细的性能报告
        PerformanceReport {
            total_execution_time: Duration::ZERO,
            node_metrics: HashMap::new(),
            custom_metrics: HashMap::new(),
            throughput: 0.0,
        }
    }
    
    fn get_resource_usage(&self) -> ResourceUsage {
        self.get_average_usage()
    }
    
    fn reset(&mut self) {
        self.resource_samples.clear();
        info!("资源监控器: 重置监控数据");
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        let node_id = NodeId::from("test_node");
        
        monitor.start_session();
        monitor.start_monitoring(&node_id);
        monitor.stop_monitoring(&node_id);
        
        let report = monitor.get_performance_report();
        assert!(report.total_execution_time > Duration::ZERO);
    }
    
    #[test]
    fn test_execution_tracer() {
        let mut tracer = ExecutionTracer::new();
        let node_id = NodeId::from("test_node");
        
        tracer.enable_tracing();
        tracer.start_monitoring(&node_id);
        tracer.stop_monitoring(&node_id);
        
        assert_eq!(tracer.get_trace().len(), 2);
    }
    
    #[test]
    fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new();
        
        monitor.start_resource_monitoring();
        monitor.sample_resources();
        monitor.stop_resource_monitoring();
        
        assert_eq!(monitor.get_resource_history().len(), 1);
    }
}