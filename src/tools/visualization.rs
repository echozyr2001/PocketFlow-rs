//! 可视化工具模块
//! 
//! 提供流程图可视化和执行过程可视化功能

use crate::core::{Store, NodeId, PostResult};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use tracing::{info, debug};

/// 可视化工具特征
pub trait VisualizationTool: Send + Sync {
    /// 生成流程图
    fn generate_flow_diagram(&self, store: &Store) -> String;
    
    /// 可视化执行路径
    fn visualize_execution_path(&self, path: &[NodeId]) -> String;
    
    /// 生成状态图
    fn generate_state_diagram(&self, states: &HashMap<String, String>) -> String;
    
    /// 导出为指定格式
    fn export(&self, content: &str, format: ExportFormat) -> Result<String, String>;
    
    /// 设置可视化选项
    fn set_options(&mut self, options: VisualizationOptions);
}

/// 导出格式
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Text,
    Dot,      // Graphviz DOT 格式
    Svg,      // SVG 格式
    Html,     // HTML 格式
    Json,     // JSON 格式
}

/// 可视化选项
#[derive(Debug, Clone)]
pub struct VisualizationOptions {
    pub show_labels: bool,
    pub show_execution_time: bool,
    pub highlight_critical_path: bool,
    pub color_scheme: ColorScheme,
    pub layout: LayoutStyle,
}

/// 颜色方案
#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    HighContrast,
    Pastel,
    Monochrome,
}

/// 布局样式
#[derive(Debug, Clone)]
pub enum LayoutStyle {
    Hierarchical,
    Circular,
    Force,
    Grid,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            show_labels: true,
            show_execution_time: false,
            highlight_critical_path: false,
            color_scheme: ColorScheme::Default,
            layout: LayoutStyle::Hierarchical,
        }
    }
}

/// 流程可视化器
/// 
/// 用于生成流程图和状态图
#[derive(Debug)]
pub struct FlowVisualizer {
    options: VisualizationOptions,
    node_positions: HashMap<NodeId, (f64, f64)>,
    connections: Vec<(NodeId, NodeId)>,
}

impl FlowVisualizer {
    pub fn new() -> Self {
        Self {
            options: VisualizationOptions::default(),
            node_positions: HashMap::new(),
            connections: Vec::new(),
        }
    }
    
    /// 设置节点位置
    pub fn set_node_position(&mut self, node_id: NodeId, x: f64, y: f64) {
        self.node_positions.insert(node_id, (x, y));
    }
    
    /// 添加连接
    pub fn add_connection(&mut self, from: NodeId, to: NodeId) {
        self.connections.push((from, to));
    }
    
    /// 生成DOT格式图表
    pub fn generate_dot(&self, store: &Store) -> String {
        let mut dot = String::new();
        dot.push_str("digraph PocketFlow {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=rounded];\n");
        
        // 添加节点
        for node_id in store.get_all_nodes() {
            let label = if self.options.show_labels {
                format!("{:?}", node_id)
            } else {
                "Node".to_string()
            };
            
            let color = self.get_node_color(&node_id);
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor=\"{}\", style=\"filled\"];\n",
                format!("{:?}", node_id), label, color
            ));
        }
        
        // 添加连接
        for (from, to) in &self.connections {
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\";\n",
                format!("{:?}", from), format!("{:?}", to)
            ));
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// 生成ASCII艺术图
    pub fn generate_ascii_art(&self, path: &[NodeId]) -> String {
        let mut art = String::new();
        art.push_str("Flow Execution Path:\n");
        art.push_str("===================\n\n");
        
        for (i, node) in path.iter().enumerate() {
            if i > 0 {
                art.push_str("   |\n");
                art.push_str("   v\n");
            }
            
            art.push_str(&format!("[{:^15}]\n", format!("{:?}", node)));
        }
        
        art
    }
    
    fn get_node_color(&self, _node_id: &NodeId) -> &str {
        match self.options.color_scheme {
            ColorScheme::Default => "lightblue",
            ColorScheme::HighContrast => "yellow",
            ColorScheme::Pastel => "lightpink",
            ColorScheme::Monochrome => "white",
        }
    }
}

impl VisualizationTool for FlowVisualizer {
    fn generate_flow_diagram(&self, store: &Store) -> String {
        match self.options.layout {
            LayoutStyle::Hierarchical | LayoutStyle::Force => self.generate_dot(store),
            _ => {
                // 对于其他布局，生成简化的文本表示
                let mut diagram = String::new();
                diagram.push_str("Flow Diagram:\n");
                for node_id in store.get_all_nodes() {
                    diagram.push_str(&format!("- {:?}\n", node_id));
                }
                diagram
            }
        }
    }
    
    fn visualize_execution_path(&self, path: &[NodeId]) -> String {
        self.generate_ascii_art(path)
    }
    
    fn generate_state_diagram(&self, states: &HashMap<String, String>) -> String {
        let mut diagram = String::new();
        diagram.push_str("State Diagram:\n");
        diagram.push_str("==============\n\n");
        
        for (key, value) in states {
            diagram.push_str(&format!("{}: {}\n", key, value));
        }
        
        diagram
    }
    
    fn export(&self, content: &str, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Text => Ok(content.to_string()),
            ExportFormat::Dot => Ok(content.to_string()),
            ExportFormat::Html => {
                let html = format!(
                    "<html><head><title>PocketFlow Visualization</title></head><body><pre>{}</pre></body></html>",
                    content
                );
                Ok(html)
            },
            ExportFormat::Json => {
                let json = format!("{{\"content\": \"{}\"}}", content.replace('\n', "\\n"));
                Ok(json)
            },
            ExportFormat::Svg => Err("SVG export not implemented".to_string()),
        }
    }
    
    fn set_options(&mut self, options: VisualizationOptions) {
        self.options = options;
        debug!("流程可视化器: 设置可视化选项");
    }
}

impl Default for FlowVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行可视化器
/// 
/// 用于可视化执行过程和性能数据
#[derive(Debug)]
pub struct ExecutionVisualizer {
    options: VisualizationOptions,
    execution_timeline: Vec<ExecutionEvent>,
    performance_data: HashMap<NodeId, PerformanceData>,
}

/// 执行事件
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    pub timestamp: std::time::Instant,
    pub node_id: NodeId,
    pub event_type: EventType,
    pub duration: Option<std::time::Duration>,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Start,
    Complete,
    Error,
    Pause,
}

/// 性能数据
#[derive(Debug, Clone)]
pub struct PerformanceData {
    pub execution_time: std::time::Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
}

impl ExecutionVisualizer {
    pub fn new() -> Self {
        Self {
            options: VisualizationOptions::default(),
            execution_timeline: Vec::new(),
            performance_data: HashMap::new(),
        }
    }
    
    /// 记录执行事件
    pub fn record_event(&mut self, node_id: NodeId, event_type: EventType) {
        let event = ExecutionEvent {
            timestamp: std::time::Instant::now(),
            node_id,
            event_type,
            duration: None,
        };
        self.execution_timeline.push(event);
    }
    
    /// 记录性能数据
    pub fn record_performance(&mut self, node_id: NodeId, data: PerformanceData) {
        self.performance_data.insert(node_id, data);
    }
    
    /// 生成时间轴图表
    pub fn generate_timeline(&self) -> String {
        let mut timeline = String::new();
        timeline.push_str("Execution Timeline:\n");
        timeline.push_str("==================\n\n");
        
        for event in &self.execution_timeline {
            timeline.push_str(&format!(
                "[{:?}] {:?} - {:?}\n",
                event.timestamp, event.node_id, event.event_type
            ));
        }
        
        timeline
    }
    
    /// 生成性能图表
    pub fn generate_performance_chart(&self) -> String {
        let mut chart = String::new();
        chart.push_str("Performance Chart:\n");
        chart.push_str("==================\n\n");
        
        for (node_id, data) in &self.performance_data {
            chart.push_str(&format!(
                "{:?}: Time={:?}, Memory={}MB, CPU={:.1}%\n",
                node_id,
                data.execution_time,
                data.memory_usage / (1024 * 1024),
                data.cpu_usage
            ));
        }
        
        chart
    }
    
    /// 生成热力图数据
    pub fn generate_heatmap_data(&self) -> HashMap<NodeId, f64> {
        let mut heatmap = HashMap::new();
        
        for (node_id, data) in &self.performance_data {
            let intensity = data.execution_time.as_millis() as f64 / 1000.0; // 以秒为单位
            heatmap.insert(node_id.clone(), intensity);
        }
        
        heatmap
    }
    
    /// 清空执行数据
    pub fn clear(&mut self) {
        self.execution_timeline.clear();
        self.performance_data.clear();
        info!("执行可视化器: 清空执行数据");
    }
}

impl VisualizationTool for ExecutionVisualizer {
    fn generate_flow_diagram(&self, _store: &Store) -> String {
        // 执行可视化器主要生成时间轴和性能图表
        self.generate_timeline()
    }
    
    fn visualize_execution_path(&self, path: &[NodeId]) -> String {
        let mut visualization = String::new();
        visualization.push_str("Execution Path Visualization:\n");
        visualization.push_str("============================\n\n");
        
        for (i, node_id) in path.iter().enumerate() {
            let performance = self.performance_data.get(node_id);
            
            if let Some(perf) = performance {
                visualization.push_str(&format!(
                    "{}. {:?} ({:?})\n",
                    i + 1, node_id, perf.execution_time
                ));
            } else {
                visualization.push_str(&format!("{}. {:?}\n", i + 1, node_id));
            }
        }
        
        visualization
    }
    
    fn generate_state_diagram(&self, states: &HashMap<String, String>) -> String {
        let mut diagram = String::new();
        diagram.push_str("Execution State Diagram:\n");
        diagram.push_str("=======================\n\n");
        
        for (key, value) in states {
            diagram.push_str(&format!("{}: {}\n", key, value));
        }
        
        diagram
    }
    
    fn export(&self, content: &str, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Text => Ok(content.to_string()),
            ExportFormat::Json => {
                let json = format!(
                    "{{\"timeline\": \"{}\", \"performance\": \"{}\"}}",
                    self.generate_timeline().replace('\n', "\\n"),
                    self.generate_performance_chart().replace('\n', "\\n")
                );
                Ok(json)
            },
            ExportFormat::Html => {
                let html = format!(
                    "<html><head><title>Execution Visualization</title></head><body><pre>{}</pre></body></html>",
                    content
                );
                Ok(html)
            },
            _ => Err("Format not supported by ExecutionVisualizer".to_string()),
        }
    }
    
    fn set_options(&mut self, options: VisualizationOptions) {
        self.options = options;
        debug!("执行可视化器: 设置可视化选项");
    }
}

impl Default for ExecutionVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flow_visualizer() {
        let mut visualizer = FlowVisualizer::new();
        let node1 = NodeId::from("node1");
        let node2 = NodeId::from("node2");
        
        visualizer.add_connection(node1.clone(), node2.clone());
        
        let path = vec![node1, node2];
        let ascii_art = visualizer.visualize_execution_path(&path);
        assert!(ascii_art.contains("node1"));
        assert!(ascii_art.contains("node2"));
    }
    
    #[test]
    fn test_execution_visualizer() {
        let mut visualizer = ExecutionVisualizer::new();
        let node_id = NodeId::from("test_node");
        
        visualizer.record_event(node_id.clone(), EventType::Start);
        visualizer.record_event(node_id.clone(), EventType::Complete);
        
        let timeline = visualizer.generate_timeline();
        assert!(timeline.contains("test_node"));
    }
    
    #[test]
    fn test_export_formats() {
        let visualizer = FlowVisualizer::new();
        let content = "test content";
        
        let text_export = visualizer.export(content, ExportFormat::Text).unwrap();
        assert_eq!(text_export, content);
        
        let html_export = visualizer.export(content, ExportFormat::Html).unwrap();
        assert!(html_export.contains("<html>"));
        assert!(html_export.contains(content));
    }
}