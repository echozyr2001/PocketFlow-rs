//! 组合模式性能测试
//! 
//! 对比不同组合策略的性能表现

use pocketflow_rs::composition::behaviors::*;
use pocketflow_rs::composition::*;
use pocketflow_rs::core::{
    communication::BaseSharedStore,
    node::NodeTrait,
    ExecResult, PrepResult, Result
};
use pocketflow_rs::compose_node;
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 组合模式性能测试 ===\n");
    
    let store = BaseSharedStore::new_in_memory();
    
    // 设置测试数据
    store.insert("performance_test", "Performance test input".to_string());
    store.insert("cache_test", "Cache test input".to_string()); 
    store.insert("retry_test", "Retry test input".to_string());
    store.insert("data_test", "Data test input".to_string());
    store.insert("input", "test_input_value".to_string());
    
    // 1. 基础组合性能
    {
        println!("1. 基础组合性能测试:");
        let start = Instant::now();
        
        let node = NodeBuilder::new()
            .with_prep(InputPrepBehavior::new("performance_test"))
            .with_exec(MockLLMExecBehavior::echo())
            .with_post(LogPostBehavior::new("performance"))
            .build();
        
        // 运行1000次
        for _ in 0..1000 {
            let _ = node.run_async(&store).await?;
        }
        
        let elapsed = start.elapsed();
        println!("   1000次执行时间: {:?}", elapsed);
        println!("   平均每次执行: {:?}\n", elapsed / 1000);
    }
    
    // 2. 缓存装饰器性能
    {
        println!("2. 缓存装饰器性能测试:");
        
        // 创建带缓存的执行器
        let exec = MockLLMExecBehavior::echo()
            .decorators()
            .with_cache()
            .build();
            
        let node = NodeBuilder::new()
            .with_prep(InputPrepBehavior::new("cache_test"))
            .with_exec(exec)
            .build();
        
        // 首次执行（无缓存）
        let start = Instant::now();
        let _ = node.run_async(&store).await?;
        let first_time = start.elapsed();
        
        // 缓存执行
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = node.run_async(&store).await?;
        }
        
        let cached_duration = start.elapsed();
        let avg_cached_time = cached_duration / iterations;
        
        println!("   首次执行时间（无缓存）: {:?}", first_time);
        println!("   {} 次缓存执行总时间: {:?}", iterations, cached_duration);
        println!("   平均缓存执行时间: {:?}", avg_cached_time);
        println!("   缓存加速比: {:.2}x\n", 
                 first_time.as_nanos() as f64 / avg_cached_time.as_nanos() as f64);
    }
    
    // 3. 简单重试机制测试
    {
        println!("3. 重试机制模拟:");
        let start = Instant::now();
        
        let node = NodeBuilder::new()
            .with_prep(InputPrepBehavior::new("retry_test"))
            .with_exec(MockLLMExecBehavior::echo())
            .build();

        let mut success_count = 0;
        for _ in 0..100 {
            match node.run_async(&store).await {
                Ok(_) => success_count += 1,
                Err(_) => {}, 
            }
        }
        
        let elapsed = start.elapsed();
        println!("   成功率: {}/100, 耗时: {:?}\n", success_count, elapsed);
    }

    // 4. 简单数据处理性能
    {
        println!("4. 数据处理性能:");
        let start = Instant::now();
        
        let node = NodeBuilder::new()
            .with_prep(InputPrepBehavior::new("data_test"))
            .with_exec(MapTransformExecBehavior::new()
                .with_mapping("input", "processed_input"))
            .with_post(LogPostBehavior::new("data_processing"))
            .build();
        
        for _ in 0..100 {
            let _ = node.run_async(&store).await?;
        }
        
        let elapsed = start.elapsed();
        println!("   数据处理100次: {:?}", elapsed);
        println!("   平均每次: {:?}\n", elapsed / 100);
    }

    // 5. 零成本抽象验证
    {
        println!("5. 零成本抽象验证:");
        
        // 直接函数调用
        let start = Instant::now();
        for _ in 0..10000 {
            let _result = json!({"direct": "call"});
        }
        let direct_time = start.elapsed();
        
        // 组合式调用
        let start = Instant::now();
        let node = compose_node! {
            exec: |_: &PrepResult| Ok(ExecResult::new(json!({"composed": "call"}))),
        };
        for _ in 0..10000 {
            let _ = node.run_async(&store).await?;
        }
        let composed_time = start.elapsed();
        
        println!("   直接调用10K次: {:?}", direct_time);
        println!("   组合调用10K次: {:?}", composed_time);
        println!("   开销比例: {:.2}x", 
                 composed_time.as_nanos() as f64 / direct_time.as_nanos() as f64);
    }
    
    println!("\n=== 性能测试完成 ===");
    Ok(())
}