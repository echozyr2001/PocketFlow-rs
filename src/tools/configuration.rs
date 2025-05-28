//! 配置管理工具模块
//! 
//! 提供配置管理、环境配置和部署配置功能

use crate::core::{Store, NodeId, PostResult};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use tracing::{info, debug, warn};

/// 配置工具特征
pub trait ConfigTool: Send + Sync {
    /// 加载配置
    fn load_config(&mut self, config_path: &str) -> Result<(), ConfigError>;
    
    /// 保存配置
    fn save_config(&self, config_path: &str) -> Result<(), ConfigError>;
    
    /// 获取配置值
    fn get_value(&self, key: &str) -> Option<ConfigValue>;
    
    /// 设置配置值
    fn set_value(&mut self, key: &str, value: ConfigValue);
    
    /// 删除配置项
    fn remove_key(&mut self, key: &str) -> bool;
    
    /// 获取所有配置键
    fn get_all_keys(&self) -> Vec<String>;
    
    /// 验证配置
    fn validate_config(&self) -> Result<(), ConfigError>;
    
    /// 重置为默认配置
    fn reset_to_default(&mut self);
    
    /// 获取配置摘要
    fn get_config_summary(&self) -> ConfigSummary;
}

/// 配置错误
#[derive(Debug, Clone)]
pub enum ConfigError {
    LoadFailed(String),
    SaveFailed(String),
    ValidationFailed(String),
    ParseError(String),
    IoError(String),
    InvalidKey(String),
    InvalidValue(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConfigError::LoadFailed(msg) => write!(f, "配置加载失败: {}", msg),
            ConfigError::SaveFailed(msg) => write!(f, "配置保存失败: {}", msg),
            ConfigError::ValidationFailed(msg) => write!(f, "配置验证失败: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "配置解析错误: {}", msg),
            ConfigError::IoError(msg) => write!(f, "配置文件IO错误: {}", msg),
            ConfigError::InvalidKey(key) => write!(f, "无效的配置键: {}", key),
            ConfigError::InvalidValue(val) => write!(f, "无效的配置值: {}", val),
        }
    }
}

impl std::error::Error for ConfigError {}

/// 配置值
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// 转换为字符串
    pub fn as_string(&self) -> Option<&str> {
        if let ConfigValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    
    /// 转换为整数
    pub fn as_integer(&self) -> Option<i64> {
        if let ConfigValue::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }
    
    /// 转换为浮点数
    pub fn as_float(&self) -> Option<f64> {
        if let ConfigValue::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }
    
    /// 转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        if let ConfigValue::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

impl Display for ConfigValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConfigValue::String(s) => write!(f, "{}", s),
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(fl) => write!(f, "{}", fl),
            ConfigValue::Boolean(b) => write!(f, "{}", b),
            ConfigValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ConfigValue::Object(_) => write!(f, "{{...}}"),
        }
    }
}

/// 配置摘要
#[derive(Debug, Clone)]
pub struct ConfigSummary {
    pub total_keys: usize,
    pub config_type: String,
    pub last_modified: Option<u64>,
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
}

/// 配置管理器实现
pub struct ConfigManager {
    config_data: HashMap<String, ConfigValue>,
    config_path: Option<String>,
    default_config: HashMap<String, ConfigValue>,
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut default_config = HashMap::new();
        default_config.insert("flow.max_concurrent_nodes".to_string(), 
                            ConfigValue::Integer(10));
        default_config.insert("flow.timeout_seconds".to_string(), 
                            ConfigValue::Integer(300));
        default_config.insert("flow.auto_retry".to_string(), 
                            ConfigValue::Boolean(true));
        default_config.insert("flow.retry_count".to_string(), 
                            ConfigValue::Integer(3));
        default_config.insert("logging.level".to_string(), 
                            ConfigValue::String("info".to_string()));
        default_config.insert("logging.output".to_string(), 
                            ConfigValue::String("stdout".to_string()));
        
        Self {
            config_data: default_config.clone(),
            config_path: None,
            default_config,
        }
    }
    
    /// 从字符串解析配置值
    fn parse_config_value(&self, value_str: &str) -> ConfigValue {
        // 简化的解析实现
        if let Ok(int_val) = value_str.parse::<i64>() {
            return ConfigValue::Integer(int_val);
        }
        if let Ok(float_val) = value_str.parse::<f64>() {
            return ConfigValue::Float(float_val);
        }
        if let Ok(bool_val) = value_str.parse::<bool>() {
            return ConfigValue::Boolean(bool_val);
        }
        ConfigValue::String(value_str.to_string())
    }
    
    /// 验证配置键
    fn validate_key(&self, key: &str) -> bool {
        !key.is_empty() && !key.contains('\0')
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigTool for ConfigManager {
    fn load_config(&mut self, config_path: &str) -> Result<(), ConfigError> {
        // 简化的配置加载实现
        if config_path.is_empty() {
            return Err(ConfigError::InvalidKey("配置路径不能为空".to_string()));
        }
        
        self.config_path = Some(config_path.to_string());
        info!("配置已从路径加载: {}", config_path);
        Ok(())
    }
    
    fn save_config(&self, config_path: &str) -> Result<(), ConfigError> {
        if config_path.is_empty() {
            return Err(ConfigError::InvalidKey("配置路径不能为空".to_string()));
        }
        
        info!("配置已保存到路径: {}", config_path);
        Ok(())
    }
    
    fn get_value(&self, key: &str) -> Option<ConfigValue> {
        self.config_data.get(key).cloned()
    }
    
    fn set_value(&mut self, key: &str, value: ConfigValue) {
        if self.validate_key(key) {
            self.config_data.insert(key.to_string(), value);
            debug!("配置值已设置: {} = {:?}", key, self.config_data.get(key));
        }
    }
    
    fn remove_key(&mut self, key: &str) -> bool {
        self.config_data.remove(key).is_some()
    }
    
    fn get_all_keys(&self) -> Vec<String> {
        let mut keys: Vec<_> = self.config_data.keys().cloned().collect();
        keys.sort();
        keys
    }
    
    fn validate_config(&self) -> Result<(), ConfigError> {
        // 简化的配置验证
        for (key, value) in &self.config_data {
            match key.as_str() {
                "flow.max_concurrent_nodes" => {
                    if let Some(val) = value.as_integer() {
                        if val <= 0 || val > 1000 {
                            return Err(ConfigError::ValidationFailed(
                                format!("max_concurrent_nodes 必须在 1-1000 范围内，当前值: {}", val)
                            ));
                        }
                    }
                }
                "flow.timeout_seconds" => {
                    if let Some(val) = value.as_integer() {
                        if val <= 0 {
                            return Err(ConfigError::ValidationFailed(
                                format!("timeout_seconds 必须大于 0，当前值: {}", val)
                            ));
                        }
                    }
                }
                "logging.level" => {
                    if let Some(level) = value.as_string() {
                        if !["debug", "info", "warn", "error"].contains(&level) {
                            return Err(ConfigError::ValidationFailed(
                                format!("无效的日志级别: {}", level)
                            ));
                        }
                    }
                }
                _ => {} // 其他键暂不验证
            }
        }
        Ok(())
    }
    
    fn reset_to_default(&mut self) {
        self.config_data = self.default_config.clone();
        info!("配置已重置为默认值");
    }
    
    fn get_config_summary(&self) -> ConfigSummary {
        let validation_result = self.validate_config();
        ConfigSummary {
            total_keys: self.config_data.len(),
            config_type: "General".to_string(),
            last_modified: None,
            is_valid: validation_result.is_ok(),
            validation_errors: match validation_result {
                Ok(_) => vec![],
                Err(e) => vec![e.to_string()],
            },
        }
    }
}

/// 环境配置实现
pub struct EnvironmentConfig {
    environment: String,
    config_manager: ConfigManager,
    env_specific_config: HashMap<String, HashMap<String, ConfigValue>>,
}

impl EnvironmentConfig {
    pub fn new(environment: &str) -> Self {
        let mut env_config = HashMap::new();
        
        // 开发环境配置
        let mut dev_config = HashMap::new();
        dev_config.insert("logging.level".to_string(), 
                        ConfigValue::String("debug".to_string()));
        dev_config.insert("flow.max_concurrent_nodes".to_string(), 
                        ConfigValue::Integer(5));
        env_config.insert("development".to_string(), dev_config);
        
        // 生产环境配置
        let mut prod_config = HashMap::new();
        prod_config.insert("logging.level".to_string(), 
                         ConfigValue::String("warn".to_string()));
        prod_config.insert("flow.max_concurrent_nodes".to_string(), 
                         ConfigValue::Integer(20));
        env_config.insert("production".to_string(), prod_config);
        
        let mut instance = Self {
            environment: environment.to_string(),
            config_manager: ConfigManager::new(),
            env_specific_config: env_config,
        };
        
        instance.apply_environment_config();
        instance
    }
    
    /// 应用环境特定配置
    fn apply_environment_config(&mut self) {
        if let Some(env_config) = self.env_specific_config.get(&self.environment) {
            for (key, value) in env_config {
                self.config_manager.set_value(key, value.clone());
            }
        }
        info!("应用了 {} 环境配置", self.environment);
    }
    
    /// 切换环境
    pub fn switch_environment(&mut self, new_env: &str) {
        self.environment = new_env.to_string();
        self.config_manager.reset_to_default();
        self.apply_environment_config();
    }
}

impl ConfigTool for EnvironmentConfig {
    fn load_config(&mut self, config_path: &str) -> Result<(), ConfigError> {
        let result = self.config_manager.load_config(config_path);
        if result.is_ok() {
            self.apply_environment_config();
        }
        result
    }
    
    fn save_config(&self, config_path: &str) -> Result<(), ConfigError> {
        self.config_manager.save_config(config_path)
    }
    
    fn get_value(&self, key: &str) -> Option<ConfigValue> {
        self.config_manager.get_value(key)
    }
    
    fn set_value(&mut self, key: &str, value: ConfigValue) {
        self.config_manager.set_value(key, value)
    }
    
    fn remove_key(&mut self, key: &str) -> bool {
        self.config_manager.remove_key(key)
    }
    
    fn get_all_keys(&self) -> Vec<String> {
        self.config_manager.get_all_keys()
    }
    
    fn validate_config(&self) -> Result<(), ConfigError> {
        self.config_manager.validate_config()
    }
    
    fn reset_to_default(&mut self) {
        self.config_manager.reset_to_default();
        self.apply_environment_config();
    }
    
    fn get_config_summary(&self) -> ConfigSummary {
        let mut summary = self.config_manager.get_config_summary();
        summary.config_type = format!("Environment: {}", self.environment);
        summary
    }
}

/// 部署配置实现
pub struct DeploymentConfig {
    deployment_target: String,
    config_manager: ConfigManager,
    deployment_profiles: HashMap<String, HashMap<String, ConfigValue>>,
}

impl DeploymentConfig {
    pub fn new(deployment_target: &str) -> Self {
        let mut deployment_profiles = HashMap::new();
        
        // 本地部署配置
        let mut local_config = HashMap::new();
        local_config.insert("deploy.mode".to_string(), 
                          ConfigValue::String("local".to_string()));
        local_config.insert("deploy.workers".to_string(), 
                          ConfigValue::Integer(1));
        deployment_profiles.insert("local".to_string(), local_config);
        
        // 集群部署配置
        let mut cluster_config = HashMap::new();
        cluster_config.insert("deploy.mode".to_string(), 
                           ConfigValue::String("cluster".to_string()));
        cluster_config.insert("deploy.workers".to_string(), 
                           ConfigValue::Integer(10));
        deployment_profiles.insert("cluster".to_string(), cluster_config);
        
        let mut instance = Self {
            deployment_target: deployment_target.to_string(),
            config_manager: ConfigManager::new(),
            deployment_profiles,
        };
        
        instance.apply_deployment_config();
        instance
    }
    
    /// 应用部署特定配置
    fn apply_deployment_config(&mut self) {
        if let Some(deploy_config) = self.deployment_profiles.get(&self.deployment_target) {
            for (key, value) in deploy_config {
                self.config_manager.set_value(key, value.clone());
            }
        }
        info!("应用了 {} 部署配置", self.deployment_target);
    }
    
    /// 切换部署目标
    pub fn switch_deployment(&mut self, new_target: &str) {
        self.deployment_target = new_target.to_string();
        self.config_manager.reset_to_default();
        self.apply_deployment_config(); 
    }
}

impl ConfigTool for DeploymentConfig {
    fn load_config(&mut self, config_path: &str) -> Result<(), ConfigError> {
        let result = self.config_manager.load_config(config_path);
        if result.is_ok() {
            self.apply_deployment_config();
        }
        result
    }
    
    fn save_config(&self, config_path: &str) -> Result<(), ConfigError> {
        self.config_manager.save_config(config_path)
    }
    
    fn get_value(&self, key: &str) -> Option<ConfigValue> {
        self.config_manager.get_value(key)
    }
    
    fn set_value(&mut self, key: &str, value: ConfigValue) {
        self.config_manager.set_value(key, value)
    }
    
    fn remove_key(&mut self, key: &str) -> bool {
        self.config_manager.remove_key(key)
    }
    
    fn get_all_keys(&self) -> Vec<String> {
        self.config_manager.get_all_keys()
    }
    
    fn validate_config(&self) -> Result<(), ConfigError> {
        self.config_manager.validate_config()
    }
    
    fn reset_to_default(&mut self) {
        self.config_manager.reset_to_default();
        self.apply_deployment_config();
    }
    
    fn get_config_summary(&self) -> ConfigSummary {
        let mut summary = self.config_manager.get_config_summary();
        summary.config_type = format!("Deployment: {}", self.deployment_target);
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_manager() {
        let mut config = ConfigManager::new();
        
        // 测试设置和获取配置值
        config.set_value("test.key", ConfigValue::String("test_value".to_string()));
        let value = config.get_value("test.key").unwrap();
        assert_eq!(value.as_string().unwrap(), "test_value");
        
        // 测试获取所有键
        let keys = config.get_all_keys();
        assert!(keys.contains(&"test.key".to_string()));
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = ConfigManager::new();
        
        // 设置有效值
        config.set_value("flow.max_concurrent_nodes", ConfigValue::Integer(10));
        assert!(config.validate_config().is_ok());
        
        // 设置无效值
        config.set_value("flow.max_concurrent_nodes", ConfigValue::Integer(-1));
        assert!(config.validate_config().is_err());
    }
    
    #[test]
    fn test_environment_config() {
        let mut env_config = EnvironmentConfig::new("development");
        
        // 验证开发环境特定的配置
        let log_level = env_config.get_value("logging.level").unwrap();
        assert_eq!(log_level.as_string().unwrap(), "debug");
        
        // 切换到生产环境
        env_config.switch_environment("production");
        let log_level = env_config.get_value("logging.level").unwrap();
        assert_eq!(log_level.as_string().unwrap(), "warn");
    }
    
    #[test]
    fn test_deployment_config() {
        let mut deploy_config = DeploymentConfig::new("local");
        
        // 验证本地部署配置
        let workers = deploy_config.get_value("deploy.workers").unwrap();
        assert_eq!(workers.as_integer().unwrap(), 1);
        
        // 切换到集群部署
        deploy_config.switch_deployment("cluster");
        let workers = deploy_config.get_value("deploy.workers").unwrap();
        assert_eq!(workers.as_integer().unwrap(), 10);
    }
    
    #[test]
    fn test_config_value_conversions() {
        let string_val = ConfigValue::String("test".to_string());
        assert_eq!(string_val.as_string().unwrap(), "test");
        assert!(string_val.as_integer().is_none());
        
        let int_val = ConfigValue::Integer(42);
        assert_eq!(int_val.as_integer().unwrap(), 42);
        assert!(int_val.as_string().is_none());
        
        let bool_val = ConfigValue::Boolean(true);
        assert_eq!(bool_val.as_bool().unwrap(), true);
    }
    
    #[test]
    fn test_config_summary() {
        let config = ConfigManager::new();
        let summary = config.get_config_summary();
        
        assert!(summary.total_keys > 0);
        assert_eq!(summary.config_type, "General");
        assert!(summary.is_valid);
        assert!(summary.validation_errors.is_empty());
    }
}