//! 组合式编程示例
//! 
//! 展示如何使用组合模式构建节点和流程

use pocketflow_rs::composition::behaviors::*;
use pocketflow_rs::composition::*;
use pocketflow_rs::core::{
    communication::{BaseSharedStore, SharedStore}, 
    node::NodeTrait,
    ExecResult, PostResult, PrepResult, Result
};
use pocketflow_rs::compose_node;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化共享存储
    let store = BaseSharedStore::new_in_memory();
    
    println!("=== 组合式编程示例 ===\n");
    
    // 示例1：最简单的组合
    simple_composition_example(&store).await?;
    
    // 示例2：使用具体行为组件的组合
    behavior_composition_example(&store).await?;
    
    // 示例3：装饰器模式组合
    decorator_composition_example(&store).await?;
    
    // 示例4：函数式组合
    functional_composition_example(&store).await?;
    
    // 示例5：宏构建
    macro_composition_example(&store).await?;
    
    Ok(())
}

/// 示例1：最简单的组合
async fn simple_composition_example(store: &BaseSharedStore) -> Result<()> {
    println!("1. 最简单的组合:");
    
    // 使用默认行为构建节点
    let node = NodeBuilder::new().build();
    
    let result = node.run_async(store).await?;
    println!("   Result: {:?}\n", result.as_str());
    
    Ok(())
}

/// 示例2：使用具体行为组件的组合  
async fn behavior_composition_example(store: &BaseSharedStore) -> Result<()> {
    println!("2. 具体行为组件组合:");
    
    // 准备输入数据
    store.insert("user_input", "Hello, how are you?".to_string());
    
    // 使用具体行为组件构建节点
    let node = NodeBuilder::new()
        .with_prep(InputPrepBehavior::new("user_input"))
        .with_exec(MockLLMExecBehavior::qa())
        .with_post(SaveResultPostBehavior::new("llm_response"))
        .build();
    
    let result = node.run_async(store).await?;
    println!("   Result: {:?}", result.as_str());
    
    // 检查保存的结果
    if let Some(response) = store.get::<serde_json::Value>("llm_response") {
        println!("   Saved Response: {}\n", response);
    }
    
    Ok(())
}

/// 示例3：装饰器模式组合
async fn decorator_composition_example(store: &BaseSharedStore) -> Result<()> {
    println!("3. 装饰器模式组合:");
    
    store.insert("retry_input", "Test retry behavior".to_string());
    
    // 创建带重试和缓存的执行行为
    let decorated_exec = MockLLMExecBehavior::echo()
        .decorators()
        .with_retry(2, 100)  // 最多重试2次，等待100ms
        .with_cache()        // 添加缓存
        .build();
    
    let node = NodeBuilder::new()
        .with_prep(InputPrepBehavior::new("retry_input"))
        .with_exec(decorated_exec)
        .with_post(LogPostBehavior::new("Input: {input}, Response: {response}"))
        .build();
    
    // 执行两次，第二次应该使用缓存
    println!("   First execution (no cache):");
    let _result1 = node.run_async(store).await?;
    
    println!("   Second execution (with cache):");
    let _result2 = node.run_async(store).await?;
    
    println!();
    Ok(())
}

/// 示例4：函数式组合
async fn functional_composition_example(store: &BaseSharedStore) -> Result<()> {
    println!("4. 函数式组合:");
    
    store.insert("math_data", json!([1, 2, 3, 4, 5]));
    
    // 使用函数式方法创建节点
    let node = node_from_fns(
        // prep: 从存储读取数组数据
        |store| {
            if let Some(data_arc) = store.get_value("math_data") {
                // 尝试将 Arc<dyn Any> 转换回 serde_json::Value
                if let Some(data) = data_arc.downcast_ref::<serde_json::Value>() {
                    Ok(PrepResult::new(json!({ "numbers": data })))
                } else {
                    Err(anyhow::anyhow!("Invalid data type"))
                }
            } else {
                Err(anyhow::anyhow!("Math data not found"))
            }
        },
        // exec: 计算数组和
        |prep_result| {
            let empty = vec![];
            let numbers = prep_result
                .get_value("numbers")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty);
                
            let sum: f64 = numbers
                .iter()
                .filter_map(|v| v.as_f64())
                .sum();
                
            Ok(ExecResult::new(json!({
                "operation": "sum",
                "result": sum,
                "count": numbers.len()
            })))
        },
        // post: 保存结果并打印
        |store, _prep, exec| {
            if let Some(result) = exec.get_value("result") {
                store.insert_value("sum_result", std::sync::Arc::new(result.clone()));
                println!("   Computed sum: {}", result);
            }
            Ok(PostResult::new("completed"))
        },
    );
    
    let result = node.run_async(store).await?;
    println!("   Final result: {:?}\n", result.as_str());
    
    Ok(())
}

/// 示例5：宏构建
async fn macro_composition_example(store: &BaseSharedStore) -> Result<()> {
    println!("5. 宏构建组合:");
    
    store.insert("message", "Macro example".to_string());
    
    // 使用宏快速构建节点
    let node = compose_node! {
        prep: |store: &dyn SharedStore| {
            if let Some(msg_arc) = store.get_value("message") {
                if let Some(msg) = msg_arc.downcast_ref::<String>() {
                    Ok(PrepResult::new(json!({ "input": msg })))
                } else {
                    Err(anyhow::anyhow!("Invalid message type"))
                }
            } else {
                Err(anyhow::anyhow!("Message not found"))
            }
        },
        exec: |prep: &PrepResult| {
            let input = prep.get_value("input")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            Ok(ExecResult::new(json!({
                "processed": format!("Processed: {}", input),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        post: |store: &dyn SharedStore, _prep: &PrepResult, exec: &ExecResult| {
            if let Some(processed) = exec.get_value("processed") {
                println!("   Macro result: {}", processed);
                store.insert_value("macro_result", std::sync::Arc::new(processed.clone()));
            }
            Ok(PostResult::new("macro_done"))
        },
    };
    
    let result = node.run_async(store).await?;
    println!("   Final result: {:?}\n", result.as_str());
    
    Ok(())
}