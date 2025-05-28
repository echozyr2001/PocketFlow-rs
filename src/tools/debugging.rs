//! 调试工具模块
//! 
//! 提供流程调试、步进执行和状态检查功能

use crate::core::{Store, NodeId, PostResult};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use tracing::{info, debug, warn};

/// 调试工具特征
pub trait DebugTool: Send + Sync {
    /// 在节点执行前设置断点
    fn set_breakpoint(&mut self, node_id: &NodeId);
    
    /// 移除断点
    fn remove_breakpoint(&mut self, node_id: &NodeId);
    
    /// 检查是否在断点处
    fn is_at_breakpoint(&self, node_id: &NodeId) -> bool;
    
    /// 单步执行到下一个节点
    fn step_next(&mut self);
    
    /// 查看当前执行状态
    fn inspect_state(&self) -> DebugState;
    
    /// 获取调试信息
    fn get_debug_info(&self) -> String;
}

/// 调试状态
#[derive(Debug, Clone)]
pub struct DebugState {
    pub current_node: Option<NodeId>,
    pub execution_path: Vec<NodeId>,
    pub breakpoints: Vec<NodeId>,
    pub variables: HashMap<String, String>,
}

impl Display for DebugState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "=== 调试状态 ===")?;
        writeln!(f, "当前节点: {:?}", self.current_node)?;
        writeln!(f, "执行路径: {:?}", self.execution_path)?;
        writeln!(f, "断点: {:?}", self.breakpoints)?;
        writeln!(f, "变量:")?;
        for (key, value) in &self.variables {
            writeln!(f, "  {}: {}", key, value)?;
        }
        Ok(())
    }
}

/// 标准调试器实现
#[derive(Debug)]
pub struct Debugger {
    breakpoints: Vec<NodeId>,
    execution_path: Vec<NodeId>,
    current_node: Option<NodeId>,
    step_mode: bool,
    variables: HashMap<String, String>,
}

impl Debugger {
    /// 创建新的调试器
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            execution_path: Vec::new(),
            current_node: None,
            step_mode: false,
            variables: HashMap::new(),
        }
    }
    
    /// 启用单步模式
    pub fn enable_step_mode(&mut self) {
        self.step_mode = true;
        info!("调试器: 启用单步模式");
    }
    
    /// 禁用单步模式
    pub fn disable_step_mode(&mut self) {
        self.step_mode = false;
        info!("调试器: 禁用单步模式");
    }
    
    /// 记录节点执行开始
    pub fn on_node_start(&mut self, node_id: NodeId) {
        self.current_node = Some(node_id.clone());
        self.execution_path.push(node_id.clone());
        debug!("调试器: 开始执行节点 {:?}", node_id);
        
        if self.step_mode || self.is_at_breakpoint(&node_id) {
            info!("调试器: 在节点 {:?} 处暂停", node_id);
        }
    }
    
    /// 记录节点执行完成
    pub fn on_node_complete(&mut self, node_id: &NodeId, result: &PostResult) {
        debug!("调试器: 节点 {:?} 执行完成，结果: {:?}", node_id, result);
        self.variables.insert(format!("result_{:?}", node_id), format!("{:?}", result));
    }
    
    /// 添加变量到调试信息
    pub fn add_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }
}

impl DebugTool for Debugger {
    fn set_breakpoint(&mut self, node_id: &NodeId) {
        if !self.breakpoints.contains(node_id) {
            self.breakpoints.push(node_id.clone());
            info!("调试器: 在节点 {:?} 设置断点", node_id);
        }
    }
    
    fn remove_breakpoint(&mut self, node_id: &NodeId) {
        if let Some(pos) = self.breakpoints.iter().position(|x| x == node_id) {
            self.breakpoints.remove(pos);
            info!("调试器: 移除节点 {:?} 的断点", node_id);
        }
    }
    
    fn is_at_breakpoint(&self, node_id: &NodeId) -> bool {
        self.breakpoints.contains(node_id)
    }
    
    fn step_next(&mut self) {
        debug!("调试器: 单步执行到下一节点");
    }
    
    fn inspect_state(&self) -> DebugState {
        DebugState {
            current_node: self.current_node.clone(),
            execution_path: self.execution_path.clone(),
            breakpoints: self.breakpoints.clone(),
            variables: self.variables.clone(),
        }
    }
    
    fn get_debug_info(&self) -> String {
        self.inspect_state().to_string()
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

/// 步进执行器
/// 
/// 专门用于单步调试执行流程
#[derive(Debug)]
pub struct StepExecutor {
    debugger: Debugger,
    paused: bool,
    next_step: bool,
}

impl StepExecutor {
    pub fn new() -> Self {
        let mut debugger = Debugger::new();
        debugger.enable_step_mode();
        
        Self {
            debugger,
            paused: false,
            next_step: false,
        }
    }
    
    /// 暂停执行
    pub fn pause(&mut self) {
        self.paused = true;
        info!("步进执行器: 执行已暂停");
    }
    
    /// 继续执行
    pub fn resume(&mut self) {
        self.paused = false;
        info!("步进执行器: 执行已恢复");
    }
    
    /// 执行下一步
    pub fn next_step(&mut self) {
        self.next_step = true;
        info!("步进执行器: 准备执行下一步");
    }
    
    /// 检查是否应该暂停
    pub fn should_pause(&mut self, node_id: &NodeId) -> bool {
        if self.paused && !self.next_step {
            return true;
        }
        
        if self.next_step {
            self.next_step = false;
            self.paused = true;
        }
        
        self.debugger.is_at_breakpoint(node_id)
    }
}

impl DebugTool for StepExecutor {
    fn set_breakpoint(&mut self, node_id: &NodeId) {
        self.debugger.set_breakpoint(node_id)
    }
    
    fn remove_breakpoint(&mut self, node_id: &NodeId) {
        self.debugger.remove_breakpoint(node_id)
    }
    
    fn is_at_breakpoint(&self, node_id: &NodeId) -> bool {
        self.debugger.is_at_breakpoint(node_id)
    }
    
    fn step_next(&mut self) {
        self.next_step()
    }
    
    fn inspect_state(&self) -> DebugState {
        self.debugger.inspect_state()
    }
    
    fn get_debug_info(&self) -> String {
        self.debugger.get_debug_info()
    }
}

impl Default for StepExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 状态检查器
/// 
/// 用于深度检查流程状态和变量
#[derive(Debug)]
pub struct StateInspector {
    snapshots: Vec<DebugState>,
    max_snapshots: usize,
}

impl StateInspector {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }
    
    pub fn with_capacity(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots,
        }
    }
    
    /// 拍摄状态快照
    pub fn take_snapshot(&mut self, state: DebugState) {
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.remove(0);
        }
        self.snapshots.push(state);
        debug!("状态检查器: 拍摄快照，当前共{}个快照", self.snapshots.len());
    }
    
    /// 获取所有快照
    pub fn get_snapshots(&self) -> &[DebugState] {
        &self.snapshots
    }
    
    /// 获取最新快照
    pub fn get_latest_snapshot(&self) -> Option<&DebugState> {
        self.snapshots.last()
    }
    
    /// 比较两个快照的差异
    pub fn compare_snapshots(&self, index1: usize, index2: usize) -> Option<String> {
        if let (Some(s1), Some(s2)) = (self.snapshots.get(index1), self.snapshots.get(index2)) {
            Some(format!(
                "快照比较 ({} vs {}):\n当前节点: {:?} -> {:?}\n执行路径长度: {} -> {}",
                index1, index2,
                s1.current_node, s2.current_node,
                s1.execution_path.len(), s2.execution_path.len()
            ))
        } else {
            None
        }
    }
}

impl DebugTool for StateInspector {
    fn set_breakpoint(&mut self, _node_id: &NodeId) {
        // StateInspector 主要用于检查，不设置断点
    }
    
    fn remove_breakpoint(&mut self, _node_id: &NodeId) {
        // StateInspector 主要用于检查，不移除断点
    }
    
    fn is_at_breakpoint(&self, _node_id: &NodeId) -> bool {
        false // StateInspector 不处理断点
    }
    
    fn step_next(&mut self) {
        // StateInspector 主要用于检查，不控制执行
    }
    
    fn inspect_state(&self) -> DebugState {
        self.get_latest_snapshot().cloned().unwrap_or_else(|| DebugState {
            current_node: None,
            execution_path: Vec::new(),
            breakpoints: Vec::new(),
            variables: HashMap::new(),
        })
    }
    
    fn get_debug_info(&self) -> String {
        format!("状态检查器: 共{}个快照", self.snapshots.len())
    }
}

impl Default for StateInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debugger() {
        let mut debugger = Debugger::new();
        let node_id = NodeId::from("test_node");
        
        debugger.set_breakpoint(&node_id);
        assert!(debugger.is_at_breakpoint(&node_id));
        
        debugger.remove_breakpoint(&node_id);
        assert!(!debugger.is_at_breakpoint(&node_id));
    }
    
    #[test]
    fn test_step_executor() {
        let mut executor = StepExecutor::new();
        let node_id = NodeId::from("test_node");
        
        executor.set_breakpoint(&node_id);
        assert!(executor.is_at_breakpoint(&node_id));
        
        executor.pause();
        assert!(executor.should_pause(&node_id));
    }
    
    #[test]
    fn test_state_inspector() {
        let mut inspector = StateInspector::new();
        
        let state = DebugState {
            current_node: Some(NodeId::from("test")),
            execution_path: vec![],
            breakpoints: vec![],
            variables: HashMap::new(),
        };
        
        inspector.take_snapshot(state);
        assert_eq!(inspector.get_snapshots().len(), 1);
    }
}