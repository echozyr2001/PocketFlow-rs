//! PocketFlow-rs 工具与实用功能模块
//! 
//! 本模块提供可选的调试、监控、可视化、持久化和配置管理工具。
//! 每个工具都基于组合设计，可以独立使用或组合使用。

pub mod debugging;
pub mod monitoring;
pub mod visualization; 
pub mod persistence;
pub mod configuration;

// 重新导出主要工具
pub use debugging::{Debugger, StepExecutor, StateInspector};
pub use monitoring::{PerformanceMonitor, ExecutionTracer, ResourceMonitor};
pub use visualization::{FlowVisualizer, ExecutionVisualizer};
pub use persistence::{StatePersister, CheckpointManager, FlowRecovery};
pub use configuration::{ConfigManager, EnvironmentConfig, DeploymentConfig};

/// 工具组合构建器
/// 
/// 允许用户灵活组合所需的工具功能
pub struct ToolsBuilder {
    debugging: Option<Box<dyn debugging::DebugTool>>,
    monitoring: Option<Box<dyn monitoring::MonitorTool>>, 
    visualization: Option<Box<dyn visualization::VisualizationTool>>,
    persistence: Option<Box<dyn persistence::PersistenceTool>>,
    configuration: Option<Box<dyn configuration::ConfigTool>>,
}

impl ToolsBuilder {
    pub fn new() -> Self {
        Self {
            debugging: None,
            monitoring: None,
            visualization: None,
            persistence: None,
            configuration: None,
        }
    }
    
    pub fn with_debugging<T: debugging::DebugTool + 'static>(mut self, debug_tool: T) -> Self {
        self.debugging = Some(Box::new(debug_tool));
        self
    }
    
    pub fn with_monitoring<T: monitoring::MonitorTool + 'static>(mut self, monitor_tool: T) -> Self {
        self.monitoring = Some(Box::new(monitor_tool));
        self
    }
    
    pub fn with_visualization<T: visualization::VisualizationTool + 'static>(mut self, viz_tool: T) -> Self {
        self.visualization = Some(Box::new(viz_tool));
        self
    }
    
    pub fn with_persistence<T: persistence::PersistenceTool + 'static>(mut self, persist_tool: T) -> Self {
        self.persistence = Some(Box::new(persist_tool));
        self
    }
    
    pub fn with_configuration<T: configuration::ConfigTool + 'static>(mut self, config_tool: T) -> Self {
        self.configuration = Some(Box::new(config_tool));
        self
    }
    
    pub fn build(self) -> ToolsComposite {
        ToolsComposite {
            debugging: self.debugging,
            monitoring: self.monitoring,
            visualization: self.visualization,
            persistence: self.persistence,
            configuration: self.configuration,
        }
    }
}

/// 工具组合容器
/// 
/// 将多个工具组合在一起，提供统一的接口
pub struct ToolsComposite {
    debugging: Option<Box<dyn debugging::DebugTool>>,
    monitoring: Option<Box<dyn monitoring::MonitorTool>>, 
    visualization: Option<Box<dyn visualization::VisualizationTool>>,
    persistence: Option<Box<dyn persistence::PersistenceTool>>,
    configuration: Option<Box<dyn configuration::ConfigTool>>,
}

impl ToolsComposite {
    /// 获取调试工具
    pub fn debugger(&self) -> Option<&dyn debugging::DebugTool> {
        self.debugging.as_ref().map(|d| d.as_ref())
    }
    
    /// 获取监控工具
    pub fn monitor(&self) -> Option<&dyn monitoring::MonitorTool> {
        self.monitoring.as_ref().map(|m| m.as_ref())
    }
    
    /// 获取可视化工具
    pub fn visualizer(&self) -> Option<&dyn visualization::VisualizationTool> {
        self.visualization.as_ref().map(|v| v.as_ref())
    }
    
    /// 获取持久化工具
    pub fn persister(&self) -> Option<&dyn persistence::PersistenceTool> {
        self.persistence.as_ref().map(|p| p.as_ref())
    }
    
    /// 获取配置工具
    pub fn config(&self) -> Option<&dyn configuration::ConfigTool> {
        self.configuration.as_ref().map(|c| c.as_ref())
    }
}

impl Default for ToolsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具组合便利宏
/// 
/// 提供快速构建工具组合的宏
#[macro_export]
macro_rules! compose_tools {
    ($($tool_type:ident($tool:expr)),* $(,)?) => {
        $crate::tools::ToolsBuilder::new()
            $(.$tool_type($tool))*
            .build()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tools_builder() {
        let tools = ToolsBuilder::new().build();
        assert!(tools.debugger().is_none());
        assert!(tools.monitor().is_none());
        assert!(tools.visualizer().is_none());
        assert!(tools.persister().is_none());
        assert!(tools.config().is_none());
    }
}