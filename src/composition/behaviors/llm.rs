//! LLM 相关行为组件

use crate::composition::behaviors::{ExecBehavior, PostBehavior, PrepBehavior};
use crate::core::{communication::SharedStore, ExecResult, PostResult, PrepResult, Result};
use async_trait::async_trait;
use serde_json::json;

/// 从存储中读取输入的准备行为
#[derive(Clone)]
pub struct InputPrepBehavior {
    pub input_key: String,
}

impl InputPrepBehavior {
    pub fn new(input_key: impl Into<String>) -> Self {
        Self {
            input_key: input_key.into(),
        }
    }
}

#[async_trait]
impl PrepBehavior for InputPrepBehavior {
    fn prep(&self, store: &dyn SharedStore) -> Result<PrepResult> {
        if let Some(input_arc) = store.get_value(&self.input_key) {
            // 尝试提取字符串值
            if let Some(input_str) = input_arc.downcast_ref::<String>() {
                return Ok(PrepResult::new(json!({
                    "input": input_str,
                    "key": self.input_key
                })));
            }
            // 尝试提取 serde_json::Value
            if let Some(input_val) = input_arc.downcast_ref::<serde_json::Value>() {
                return Ok(PrepResult::new(json!({
                    "input": input_val,
                    "key": self.input_key
                })));
            }
        }
        
        // 如果没有找到输入，返回错误
        Err(anyhow::anyhow!("Input not found in store with key: {}", self.input_key))
    }
}

/// 模拟 LLM 调用的执行行为
#[derive(Clone)]
pub struct MockLLMExecBehavior {
    pub response_template: String,
}

impl MockLLMExecBehavior {
    pub fn new(response_template: impl Into<String>) -> Self {
        Self {
            response_template: response_template.into(),
        }
    }
    
    /// 创建一个简单的回显 LLM 行为
    pub fn echo() -> Self {
        Self::new("Echo: {input}")
    }
    
    /// 创建一个问答 LLM 行为
    pub fn qa() -> Self {
        Self::new("Answer: This is a response to '{input}'")
    }
}

#[async_trait]
impl ExecBehavior for MockLLMExecBehavior {
    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        let input = prep_result
            .get_value("input")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown input");
            
        let response = self.response_template.replace("{input}", input);
        
        Ok(ExecResult::new(json!({
            "response": response,
            "input": input,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    }

    async fn exec_async(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        // 模拟异步调用延迟
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.exec(prep_result)
    }
}

/// 将结果保存到存储的后处理行为
#[derive(Clone)]
pub struct SaveResultPostBehavior {
    pub output_key: String,
    pub next_action: String,
}

impl SaveResultPostBehavior {
    pub fn new(output_key: impl Into<String>) -> Self {
        Self {
            output_key: output_key.into(),
            next_action: "default".to_string(),
        }
    }
    
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.next_action = action.into();
        self
    }
}

#[async_trait]
impl PostBehavior for SaveResultPostBehavior {
    fn post(
        &self,
        store: &dyn SharedStore,
        _prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<PostResult> {
        // 保存执行结果
        if let Some(response) = exec_result.get_value("response") {
            store.insert_value(&self.output_key, std::sync::Arc::new(response.clone()));
        } else {
            // 如果没有response字段，保存整个结果
            store.insert_value(&self.output_key, std::sync::Arc::new(exec_result.clone()));
        }
        
        Ok(PostResult::new(&self.next_action))
    }
}

/// 简单的日志后处理行为
#[derive(Clone)]
pub struct LogPostBehavior {
    pub message_template: String,
    pub next_action: String,
}

impl LogPostBehavior {
    pub fn new(message_template: impl Into<String>) -> Self {
        Self {
            message_template: message_template.into(),
            next_action: "default".to_string(),
        }
    }
    
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.next_action = action.into();
        self
    }
}

#[async_trait]
impl PostBehavior for LogPostBehavior {
    fn post(
        &self,
        _store: &dyn SharedStore,
        prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<PostResult> {
        // 简单的日志输出
        let input = prep_result
            .get_value("input")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let response = exec_result
            .get_value("response")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let message = self.message_template
            .replace("{input}", input)
            .replace("{response}", response);
            
        println!("[LOG] {}", message);
        
        Ok(PostResult::new(&self.next_action))
    }
}