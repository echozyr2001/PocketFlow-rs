//! 数据处理相关行为组件

use crate::composition::behaviors::ExecBehavior;
use crate::core::{ExecResult, PrepResult, Result};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

/// 键值映射转换执行行为
#[derive(Clone)]
pub struct MapTransformExecBehavior {
    pub transformations: HashMap<String, String>,
}

impl MapTransformExecBehavior {
    pub fn new() -> Self {
        Self {
            transformations: HashMap::new(),
        }
    }
    
    pub fn with_mapping(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.transformations.insert(from.into(), to.into());
        self
    }
    
    pub fn from_mappings(transformations: HashMap<String, String>) -> Self {
        Self { transformations }
    }
}

impl Default for MapTransformExecBehavior {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecBehavior for MapTransformExecBehavior {
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let mut result_data = json!({});
        
        // 遍历输入数据
        if let Some(input_obj) = prep_result.as_object() {
            for (key, value) in input_obj {
                let output_key = self.transformations.get(key).unwrap_or(key);
                result_data[output_key] = value.clone();
            }
        }
        
        Ok(ExecResult::new(result_data))
    }
}

/// 数据聚合执行行为
#[derive(Clone)]
pub struct AggregateExecBehavior {
    pub operation: String, // "sum", "count", "concat", etc.
    pub field: String,
}

impl AggregateExecBehavior {
    pub fn sum(field: impl Into<String>) -> Self {
        Self {
            operation: "sum".to_string(),
            field: field.into(),
        }
    }
    
    pub fn count(field: impl Into<String>) -> Self {
        Self {
            operation: "count".to_string(),
            field: field.into(),
        }
    }
    
    pub fn concat(field: impl Into<String>) -> Self {
        Self {
            operation: "concat".to_string(),
            field: field.into(),
        }
    }
}

#[async_trait]
impl ExecBehavior for AggregateExecBehavior {
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let data = prep_result.get_value(&self.field);
        
        let result = match self.operation.as_str() {
            "sum" => {
                if let Some(array) = data.and_then(|v| v.as_array()) {
                    let sum: f64 = array
                        .iter()
                        .filter_map(|v| v.as_f64())
                        .sum();
                    json!(sum)
                } else {
                    json!(0.0)
                }
            }
            "count" => {
                if let Some(array) = data.and_then(|v| v.as_array()) {
                    json!(array.len())
                } else {
                    json!(0)
                }
            }
            "concat" => {
                if let Some(array) = data.and_then(|v| v.as_array()) {
                    let concat: String = array
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("");
                    json!(concat)
                } else {
                    json!("")
                }
            }
            _ => json!(null),
        };
        
        Ok(ExecResult::new(json!({
            "operation": self.operation,
            "field": self.field,
            "result": result
        })))
    }
}