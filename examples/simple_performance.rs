//! 简化性能测试

use pocketflow_rs::composition::behaviors::*;
use pocketflow_rs::composition::*;
use pocketflow_rs::core::{
    Result,
    communication::{BaseSharedStore, SharedStore},
    node::NodeTrait,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 简化性能测试 ===\n");

    let store = BaseSharedStore::new_in_memory();

    // 设置数据
    store.insert("test_input", "Hello World!".to_string());
    println!("Data inserted. Keys: {:?}", store.keys());

    // 测试简单的 LLM 节点
    {
        println!("基础测试:");
        let start = Instant::now();

        let node = NodeBuilder::new()
            .with_prep(InputPrepBehavior::new("test_input"))
            .with_exec(MockLLMExecBehavior::echo())
            .build();

        println!("运行 10 次:");
        for i in 0..10 {
            match node.run_async(&store).await {
                Ok(_result) => println!("  Run {}: Success", i + 1),
                Err(e) => println!("  Run {}: Error - {}", i + 1, e),
            }
        }

        let elapsed = start.elapsed();
        println!("总时间: {:?}\n", elapsed);
    }

    Ok(())
}
