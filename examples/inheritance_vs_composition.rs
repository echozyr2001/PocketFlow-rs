//! Python继承 vs Rust组合 对比示例
//! 
//! 展示如何从继承思维转向组合思维

use pocketflow_rs::composition::behaviors::*;
use pocketflow_rs::composition::*;
use pocketflow_rs::core::{
    communication::{BaseSharedStore, SharedStore}, 
    node::NodeTrait, 
    ExecResult, PrepResult, Result
};
use serde_json::json;
use std::sync::Arc;

/// 模拟传统继承式节点
#[derive(Clone)]
struct TraditionalLLMNode {
    prompt_template: String,
    model: String,
}

impl TraditionalLLMNode {
    fn new(prompt_template: &str, model: &str) -> Self {
        Self {
            prompt_template: prompt_template.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl NodeTrait for TraditionalLLMNode {
    fn prep(&self, store: &dyn pocketflow_rs::core::communication::SharedStore) -> Result<pocketflow_rs::core::PrepResult> {
        if let Some(input_arc) = store.get_value("input") {
            let input_str = if let Some(s) = input_arc.downcast_ref::<String>() {
                s.as_str()
            } else if let Some(v) = input_arc.downcast_ref::<serde_json::Value>() {
                v.as_str().unwrap_or("")
            } else {
                ""
            };
            
            let formatted_prompt = self.prompt_template
                .replace("{input}", input_str);
                
            Ok(pocketflow_rs::core::PrepResult::new(json!({
                "prompt": formatted_prompt,
                "model": self.model
            })))
        } else {
            Err(anyhow::anyhow!("Input not found"))
        }
    }

    fn exec(&self, prep_res: &pocketflow_rs::core::PrepResult) -> Result<pocketflow_rs::core::ExecResult> {
        let prompt = prep_res.get_value("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // 模拟LLM调用
        let response = format!("AI Response to: {}", prompt);
        
        Ok(pocketflow_rs::core::ExecResult::new(json!({
            "response": response,
            "model": self.model,
            "tokens": prompt.len() * 2
        })))
    }

    fn post(
        &self,
        store: &dyn pocketflow_rs::core::communication::SharedStore,
        _prep_res: &pocketflow_rs::core::PrepResult,
        exec_res: &pocketflow_rs::core::ExecResult,
    ) -> Result<pocketflow_rs::core::PostResult> {
        if let Some(response) = exec_res.get_value("response") {
            store.insert_value("ai_response", std::sync::Arc::new(response.clone()));
        }
        Ok(pocketflow_rs::core::PostResult::new("default"))
    }

    async fn prep_async(&self, store: &dyn pocketflow_rs::core::communication::SharedStore) -> Result<pocketflow_rs::core::PrepResult> {
        self.prep(store)
    }

    async fn exec_async(&self, prep_res: &pocketflow_rs::core::PrepResult) -> Result<pocketflow_rs::core::ExecResult> {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // 模拟异步延迟
        self.exec(prep_res)
    }

    async fn post_async(
        &self,
        store: &dyn pocketflow_rs::core::communication::SharedStore,
        prep_res: &pocketflow_rs::core::PrepResult,
        exec_res: &pocketflow_rs::core::ExecResult,
    ) -> Result<pocketflow_rs::core::PostResult> {
        self.post(store, prep_res, exec_res)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let store = BaseSharedStore::new_in_memory();
    store.insert("input", "What is Rust programming language?".to_string());
    
    println!("=== Python继承 vs Rust组合 对比 ===\n");
    
    // 1. 传统继承式方式 
    println!("1. 传统继承式方式 (类似Python类继承):");
    println!("   - 需要实现完整的NodeTrait");
    println!("   - 每个方法都要重新实现");
    println!("   - 代码重复，难以复用");
    
    let traditional_node = TraditionalLLMNode::new(
        "You are a helpful assistant. User asks: {input}", 
        "gpt-3.5-turbo"
    );
    
    let start = std::time::Instant::now();
    let result1 = traditional_node.run_async(&store).await?;
    let duration1 = start.elapsed();
    
    println!("   执行结果: {:?}", result1.as_str());
    println!("   执行时间: {:?}", duration1);
    println!();
    
    // 2. 组合式方式
    println!("2. Rust组合式方式 (组合行为组件):");
    println!("   - 通过组合预定义的行为组件");
    println!("   - 每个组件职责单一，高度复用");
    println!("   - 灵活的装饰器模式");
    
    // 定义自定义的模板准备行为
    let template_prep = FnPrepBehavior::new(|store: &dyn SharedStore| {
        if let Some(input_arc) = store.get_value("input") {
            let input_str = if let Some(s) = input_arc.downcast_ref::<String>() {
                s.as_str()
            } else if let Some(v) = input_arc.downcast_ref::<serde_json::Value>() {
                v.as_str().unwrap_or("")
            } else {
                ""
            };
            
            let template = "You are a helpful assistant. User asks: {input}";
            let formatted_prompt = template.replace("{input}", input_str);
            Ok(PrepResult::new(json!({
                "prompt": formatted_prompt,
                "model": "gpt-3.5-turbo"
            })))
        } else {
            Err(anyhow::anyhow!("Input not found"))
        }
    });
    
    // 组合式LLM执行行为
    let llm_exec = FnExecBehavior::new(|prep_res: &PrepResult| {
        let prompt = prep_res.get_value("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let model = prep_res.get_value("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let response = format!("AI Response to: {}", prompt);
        
        Ok(ExecResult::new(json!({
            "response": response,
            "model": model,
            "tokens": prompt.len() * 2
        })))
    });
    
    // 带缓存和重试的装饰器
    let decorated_exec = llm_exec
        .decorators()
        .with_cache()
        .with_retry(2, 50)
        .build();
    
    let composition_node = NodeBuilder::new()
        .with_prep(template_prep)
        .with_exec(decorated_exec)
        .with_post(SaveResultPostBehavior::new("ai_response"))
        .build();
    
    let start = std::time::Instant::now();
    let result2 = composition_node.run_async(&store).await?;
    let duration2 = start.elapsed();
    
    println!("   執行結果: {:?}", result2.as_str());
    println!("   執行時間: {:?}", duration2);
    println!();
    
    // 3. 展示组合的优势
    println!("3. 组合模式的优势演示:");
    println!("   - 可以轻松替换组件:");
    
    // 替换执行行为为聊天模式
    let chat_node = composition_node
        .with_exec_behavior(Arc::new(MockLLMExecBehavior::new("Chat: {input}")));
    
    let result3 = chat_node.run_async(&store).await?;
    println!("     替换后的结果: {:?}", result3.as_str());
    
    // 4. 性能和灵活性对比
    println!("\n4. 对比总结:");
    println!("   传统继承式:");
    println!("   ✗ 代码重复多");
    println!("   ✗ 难以组件复用");
    println!("   ✗ 扩展需要修改原类");
    println!("   ✓ 概念简单直接");
    
    println!("\n   Rust组合式:");
    println!("   ✓ 组件高度复用");
    println!("   ✓ 零成本抽象");
    println!("   ✓ 类型安全保障");
    println!("   ✓ 装饰器模式增强");
    println!("   ✓ 运行时可替换");
    println!("   ▲ 初期学习成本高");
    
    Ok(())
}