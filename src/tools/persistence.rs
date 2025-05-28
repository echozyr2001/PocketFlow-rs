//! 持久化工具模块
//! 
//! 提供状态持久化、检查点管理和流程恢复功能

use crate::core::{Store, NodeId, PostResult};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{Display, Formatter, Result as FmtResult};
use tracing::{info, debug, warn};

/// 持久化工具特征
pub trait PersistenceTool: Send + Sync {
    /// 保存当前状态
    fn save_state(&mut self, checkpoint_name: &str) -> Result<String, PersistenceError>;
    
    /// 加载状态
    fn load_state(&mut self, checkpoint_id: &str) -> Result<FlowState, PersistenceError>;
    
    /// 列出所有检查点
    fn list_checkpoints(&self) -> Vec<CheckpointInfo>;
    
    /// 删除检查点
    fn delete_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), PersistenceError>;
    
    /// 自动检查点（定期保存）
    fn enable_auto_checkpoint(&mut self, interval_secs: u64);
    
    /// 禁用自动检查点
    fn disable_auto_checkpoint(&mut self);
    
    /// 获取持久化统计信息
    fn get_stats(&self) -> PersistenceStats;
}

/// 持久化错误
#[derive(Debug, Clone)]
pub enum PersistenceError {
    SaveFailed(String),
    LoadFailed(String),
    CheckpointNotFound(String),
    SerializationError(String),
    IoError(String),
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PersistenceError::SaveFailed(msg) => write!(f, "保存失败: {}", msg),
            PersistenceError::LoadFailed(msg) => write!(f, "加载失败: {}", msg),
            PersistenceError::CheckpointNotFound(id) => write!(f, "检查点未找到: {}", id),
            PersistenceError::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            PersistenceError::IoError(msg) => write!(f, "IO错误: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

/// 流程状态
#[derive(Debug, Clone)]
pub struct FlowState {
    pub node_states: HashMap<NodeId, NodeState>,
    pub global_context: HashMap<String, String>,
    pub execution_path: Vec<NodeId>,
    pub timestamp: u64,
}

/// 节点状态
#[derive(Debug, Clone)]
pub struct NodeState {
    pub node_id: NodeId,
    pub status: NodeStatus,
    pub input_data: Option<String>,
    pub output_data: Option<String>,
    pub error: Option<String>,
}

/// 节点状态枚举
#[derive(Debug, Clone)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// 检查点信息
#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub id: String,
    pub name: String,
    pub timestamp: u64,
    pub size_bytes: u64,
    pub node_count: usize,
}

/// 持久化统计信息
#[derive(Debug, Clone)]
pub struct PersistenceStats {
    pub total_saves: u64,
    pub total_loads: u64,
    pub total_checkpoints: usize,
    pub total_storage_size: u64,
    pub auto_checkpoint_enabled: bool,
    pub last_save_time: Option<u64>,
}

/// 状态持久化器实现
pub struct StatePersister {
    checkpoints: HashMap<String, FlowState>,
    checkpoint_info: HashMap<String, CheckpointInfo>,
    auto_checkpoint_interval: Option<u64>,
    stats: PersistenceStats,
    last_checkpoint_time: Option<SystemTime>,
}

impl StatePersister {
    pub fn new() -> Self {
        Self {
            checkpoints: HashMap::new(),
            checkpoint_info: HashMap::new(),
            auto_checkpoint_interval: None,
            stats: PersistenceStats {
                total_saves: 0,
                total_loads: 0,
                total_checkpoints: 0,
                total_storage_size: 0,
                auto_checkpoint_enabled: false,
                last_save_time: None,
            },
            last_checkpoint_time: None,
        }
    }
    
    /// 创建检查点ID
    fn create_checkpoint_id(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("checkpoint_{}", timestamp)
    }
    
    /// 计算状态大小（简化实现）
    fn calculate_state_size(&self, state: &FlowState) -> u64 {
        (state.node_states.len() * 100 + state.global_context.len() * 50) as u64
    }
}

impl Default for StatePersister {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceTool for StatePersister {
    fn save_state(&mut self, checkpoint_name: &str) -> Result<String, PersistenceError> {
        let checkpoint_id = self.create_checkpoint_id();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 创建模拟的流程状态
        let flow_state = FlowState {
            node_states: HashMap::new(),
            global_context: HashMap::new(),
            execution_path: Vec::new(),
            timestamp,
        };
        
        let size = self.calculate_state_size(&flow_state);
        
        let checkpoint_info = CheckpointInfo {
            id: checkpoint_id.clone(),
            name: checkpoint_name.to_string(),
            timestamp,
            size_bytes: size,
            node_count: flow_state.node_states.len(),
        };
        
        self.checkpoints.insert(checkpoint_id.clone(), flow_state);
        self.checkpoint_info.insert(checkpoint_id.clone(), checkpoint_info);
        
        self.stats.total_saves += 1;
        self.stats.total_checkpoints = self.checkpoints.len();
        self.stats.total_storage_size += size;
        self.stats.last_save_time = Some(timestamp);
        
        info!("状态已保存到检查点: {} (ID: {})", checkpoint_name, checkpoint_id);
        Ok(checkpoint_id)
    }
    
    fn load_state(&mut self, checkpoint_id: &str) -> Result<FlowState, PersistenceError> {
        match self.checkpoints.get(checkpoint_id) {
            Some(state) => {
                self.stats.total_loads += 1;
                info!("状态已从检查点加载: {}", checkpoint_id);
                Ok(state.clone())
            }
            None => Err(PersistenceError::CheckpointNotFound(checkpoint_id.to_string()))
        }
    }
    
    fn list_checkpoints(&self) -> Vec<CheckpointInfo> {
        let mut checkpoints: Vec<_> = self.checkpoint_info.values().cloned().collect();
        checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        checkpoints
    }
    
    fn delete_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), PersistenceError> {
        if let Some(checkpoint) = self.checkpoints.remove(checkpoint_id) {
            self.checkpoint_info.remove(checkpoint_id);
            let size = self.calculate_state_size(&checkpoint);
            self.stats.total_storage_size = self.stats.total_storage_size.saturating_sub(size);
            self.stats.total_checkpoints = self.checkpoints.len();
            info!("检查点已删除: {}", checkpoint_id);
            Ok(())
        } else {
            Err(PersistenceError::CheckpointNotFound(checkpoint_id.to_string()))
        }
    }
    
    fn enable_auto_checkpoint(&mut self, interval_secs: u64) {
        self.auto_checkpoint_interval = Some(interval_secs);
        self.stats.auto_checkpoint_enabled = true;
        info!("启用自动检查点，间隔: {} 秒", interval_secs);
    }
    
    fn disable_auto_checkpoint(&mut self) {
        self.auto_checkpoint_interval = None;
        self.stats.auto_checkpoint_enabled = false;
        info!("禁用自动检查点");
    }
    
    fn get_stats(&self) -> PersistenceStats {
        self.stats.clone()
    }
}

/// 检查点管理器实现
pub struct CheckpointManager {
    persister: StatePersister,
    max_checkpoints: usize,
}

impl CheckpointManager {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            persister: StatePersister::new(),
            max_checkpoints,
        }
    }
    
    /// 清理旧检查点
    fn cleanup_old_checkpoints(&mut self) {
        if self.persister.checkpoints.len() > self.max_checkpoints {
            let mut checkpoints = self.persister.list_checkpoints();
            checkpoints.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            
            let to_remove = checkpoints.len() - self.max_checkpoints;
            for checkpoint in checkpoints.iter().take(to_remove) {
                let _ = self.persister.delete_checkpoint(&checkpoint.id);
            }
        }
    }
}

impl PersistenceTool for CheckpointManager {
    fn save_state(&mut self, checkpoint_name: &str) -> Result<String, PersistenceError> {
        let result = self.persister.save_state(checkpoint_name);
        if result.is_ok() {
            self.cleanup_old_checkpoints();
        }
        result
    }
    
    fn load_state(&mut self, checkpoint_id: &str) -> Result<FlowState, PersistenceError> {
        self.persister.load_state(checkpoint_id)
    }
    
    fn list_checkpoints(&self) -> Vec<CheckpointInfo> {
        self.persister.list_checkpoints()
    }
    
    fn delete_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), PersistenceError> {
        self.persister.delete_checkpoint(checkpoint_id)
    }
    
    fn enable_auto_checkpoint(&mut self, interval_secs: u64) {
        self.persister.enable_auto_checkpoint(interval_secs)
    }
    
    fn disable_auto_checkpoint(&mut self) {
        self.persister.disable_auto_checkpoint()
    }
    
    fn get_stats(&self) -> PersistenceStats {
        self.persister.get_stats()
    }
}

/// 流程恢复工具实现
pub struct FlowRecovery {
    persister: StatePersister,
    recovery_strategy: RecoveryStrategy,
}

/// 恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 恢复到最近的检查点
    LatestCheckpoint,
    /// 恢复到指定检查点
    SpecificCheckpoint(String),
    /// 恢复到最近的成功状态
    LastSuccessful,
}

impl FlowRecovery {
    pub fn new(strategy: RecoveryStrategy) -> Self {
        Self {
            persister: StatePersister::new(),
            recovery_strategy: strategy,
        }
    }
    
    /// 自动恢复
    pub fn auto_recover(&mut self) -> Result<FlowState, PersistenceError> {
        match &self.recovery_strategy {
            RecoveryStrategy::LatestCheckpoint => {
                let checkpoints = self.persister.list_checkpoints();
                if let Some(latest) = checkpoints.first() {
                    self.persister.load_state(&latest.id)
                } else {
                    Err(PersistenceError::CheckpointNotFound("无可用检查点".to_string()))
                }
            }
            RecoveryStrategy::SpecificCheckpoint(id) => {
                self.persister.load_state(id)
            }
            RecoveryStrategy::LastSuccessful => {
                // 简化实现：返回最新检查点
                let checkpoints = self.persister.list_checkpoints();
                if let Some(latest) = checkpoints.first() {
                    self.persister.load_state(&latest.id)
                } else {
                    Err(PersistenceError::CheckpointNotFound("无可用检查点".to_string()))
                }
            }
        }
    }
}

impl PersistenceTool for FlowRecovery {
    fn save_state(&mut self, checkpoint_name: &str) -> Result<String, PersistenceError> {
        self.persister.save_state(checkpoint_name)
    }
    
    fn load_state(&mut self, checkpoint_id: &str) -> Result<FlowState, PersistenceError> {
        self.persister.load_state(checkpoint_id)
    }
    
    fn list_checkpoints(&self) -> Vec<CheckpointInfo> {
        self.persister.list_checkpoints()
    }
    
    fn delete_checkpoint(&mut self, checkpoint_id: &str) -> Result<(), PersistenceError> {
        self.persister.delete_checkpoint(checkpoint_id)
    }
    
    fn enable_auto_checkpoint(&mut self, interval_secs: u64) {
        self.persister.enable_auto_checkpoint(interval_secs)
    }
    
    fn disable_auto_checkpoint(&mut self) {
        self.persister.disable_auto_checkpoint()
    }
    
    fn get_stats(&self) -> PersistenceStats {
        self.persister.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_persister() {
        let mut persister = StatePersister::new();
        
        // 测试保存状态
        let checkpoint_id = persister.save_state("test_checkpoint").unwrap();
        assert!(!checkpoint_id.is_empty());
        
        // 测试加载状态
        let state = persister.load_state(&checkpoint_id).unwrap();
        assert_eq!(state.node_states.len(), 0);
        
        // 测试列出检查点
        let checkpoints = persister.list_checkpoints();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].name, "test_checkpoint");
    }
    
    #[test]
    fn test_checkpoint_manager() {
        let mut manager = CheckpointManager::new(2);
        
        // 创建多个检查点
        manager.save_state("checkpoint1").unwrap();
        manager.save_state("checkpoint2").unwrap();
        manager.save_state("checkpoint3").unwrap();
        
        // 检查是否自动清理了旧检查点
        let checkpoints = manager.list_checkpoints();
        assert!(checkpoints.len() <= 2);
    }
    
    #[test]
    fn test_flow_recovery() {
        let mut recovery = FlowRecovery::new(RecoveryStrategy::LatestCheckpoint);
        
        // 保存检查点
        let checkpoint_id = recovery.save_state("recovery_test").unwrap();
        
        // 测试自动恢复
        let state = recovery.auto_recover().unwrap();
        assert_eq!(state.node_states.len(), 0);
    }
    
    #[test]
    fn test_auto_checkpoint() {
        let mut persister = StatePersister::new();
        
        // 启用自动检查点
        persister.enable_auto_checkpoint(60);
        assert!(persister.get_stats().auto_checkpoint_enabled);
        
        // 禁用自动检查点
        persister.disable_auto_checkpoint();
        assert!(!persister.get_stats().auto_checkpoint_enabled);
    }
    
    #[test]
    fn test_checkpoint_deletion() {
        let mut persister = StatePersister::new();
        
        let checkpoint_id = persister.save_state("to_delete").unwrap();
        assert_eq!(persister.list_checkpoints().len(), 1);
        
        persister.delete_checkpoint(&checkpoint_id).unwrap();
        assert_eq!(persister.list_checkpoints().len(), 0);
        
        // 测试删除不存在的检查点
        let result = persister.delete_checkpoint("nonexistent");
        assert!(result.is_err());
    }
}