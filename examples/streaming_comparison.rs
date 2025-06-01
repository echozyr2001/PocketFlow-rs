use pocketflow_rs::{
    Action, ExecutionContext, InMemoryStorage, SharedStore,
    node::NodeBackend,
    node::builtin::llm::{ApiConfig, ApiRequestNode},
};
use serde_json::json;
use std::time::Instant;
use tokio;

/// 流式 vs 非流式 API 对比演示
/// 展示两种模式在性能和用户体验上的差异
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚖️  流式 vs 非流式 API 对比演示");
    println!("================================");

    // 检查API密钥
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  使用演示密钥，实际API调用将失败");
        "demo_key".to_string()
    });

    let mut demo = ComparisonDemo::new(api_key).await?;

    // 运行对比测试
    demo.run_comparison_tests().await?;

    Ok(())
}

pub struct ComparisonDemo {
    store: SharedStore<InMemoryStorage>,
    execution_context: ExecutionContext,
    streaming_node: ApiRequestNode,
    regular_node: ApiRequestNode,
}

impl ComparisonDemo {
    pub async fn new(api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        let store: SharedStore<InMemoryStorage> = SharedStore::new();
        let execution_context = ExecutionContext::new(3, std::time::Duration::from_secs(60));

        // 流式配置
        let streaming_config = ApiConfig {
            api_key: api_key.clone(),
            base_url: None,
            org_id: None,
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: Some(300),
            temperature: Some(0.7),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            timeout: Some(30),
            stream: true, // 启用流式
        };

        // 非流式配置
        let regular_config = ApiConfig {
            stream: false, // 禁用流式
            ..streaming_config.clone()
        };

        let streaming_node = ApiRequestNode::new("input", "output", Action::simple("next"))
            .with_config(streaming_config);

        let regular_node = ApiRequestNode::new("input", "output", Action::simple("next"))
            .with_config(regular_config);

        Ok(Self {
            store,
            execution_context,
            streaming_node,
            regular_node,
        })
    }

    pub async fn run_comparison_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let test_queries = vec![
            "请简单解释什么是机器学习",
            "写一个Python的Hello World程序并解释",
            "什么是区块链技术？请用通俗的语言解释",
            "解释一下什么是递归算法，并举个例子",
        ];

        for (i, query) in test_queries.iter().enumerate() {
            println!("\n{}", "=".repeat(60));
            println!("🧪 测试 {} / {}", i + 1, test_queries.len());
            println!("❓ 问题: {}", query);
            println!("{}", "=".repeat(60));

            // 设置输入
            self.store.set("input".to_string(), json!(query))?;

            // 测试流式模式
            println!("\n🔄 **流式模式测试**");
            let streaming_result = self.test_streaming_node(query).await;

            println!("\n{}", "-".repeat(40));

            // 测试非流式模式
            println!("\n📦 **非流式模式测试**");
            let regular_result = self.test_regular_node(query).await;

            // 显示对比结果
            self.show_comparison(streaming_result, regular_result);

            // 如果不是最后一个测试，等待用户确认
            if i < test_queries.len() - 1 {
                println!("\n⏸️  按 Enter 继续下一个测试...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }
        }

        self.show_summary();
        Ok(())
    }

    /// 测试流式节点
    async fn test_streaming_node(&mut self, query: &str) -> ApiCallResult {
        // 设置输入
        if let Err(_) = self.store.set("input".to_string(), json!(query)) {
            return ApiCallResult {
                mode: "流式".to_string(),
                success: false,
                duration: std::time::Duration::ZERO,
                response_length: 0,
                error_message: Some("输入设置失败".to_string()),
            };
        }

        let start_time = Instant::now();
        let mut result = ApiCallResult {
            mode: "流式".to_string(),
            success: false,
            duration: std::time::Duration::ZERO,
            response_length: 0,
            error_message: None,
        };

        println!("⏱️  开始时间: {:?}", start_time);

        match self
            .streaming_node
            .prep(&self.store, &self.execution_context)
            .await
        {
            Ok(messages) => {
                println!("✅ 消息准备完成: {} 条", messages.len());

                match <ApiRequestNode as NodeBackend<InMemoryStorage>>::exec(
                    &mut self.streaming_node,
                    messages,
                    &self.execution_context,
                )
                .await
                {
                    Ok(response) => {
                        result.duration = start_time.elapsed();
                        result.success = true;
                        result.response_length = response.len();

                        println!("✅ 响应成功!");
                        println!("📏 响应长度: {} 字符", response.len());
                        println!("⏱️  总耗时: {:.2?}", result.duration);

                        // 显示响应的前100个字符
                        let preview = if response.len() > 100 {
                            format!("{}...", &response[..100])
                        } else {
                            response
                        };
                        println!("📝 响应预览: {}", preview);
                    }
                    Err(e) => {
                        result.duration = start_time.elapsed();
                        result.error_message = Some(e.to_string());

                        println!("❌ 执行失败: {}", e);
                        println!("⏱️  总耗时: {:.2?}", result.duration);
                    }
                }
            }
            Err(e) => {
                result.duration = start_time.elapsed();
                result.error_message = Some(e.to_string());

                println!("❌ 消息准备失败: {}", e);
                println!("⏱️  总耗时: {:.2?}", result.duration);
            }
        }

        result
    }

    /// 测试非流式节点
    async fn test_regular_node(&mut self, query: &str) -> ApiCallResult {
        // 设置输入
        if let Err(_) = self.store.set("input".to_string(), json!(query)) {
            return ApiCallResult {
                mode: "非流式".to_string(),
                success: false,
                duration: std::time::Duration::ZERO,
                response_length: 0,
                error_message: Some("输入设置失败".to_string()),
            };
        }

        let start_time = Instant::now();
        let mut result = ApiCallResult {
            mode: "非流式".to_string(),
            success: false,
            duration: std::time::Duration::ZERO,
            response_length: 0,
            error_message: None,
        };

        println!("⏱️  开始时间: {:?}", start_time);

        match self
            .regular_node
            .prep(&self.store, &self.execution_context)
            .await
        {
            Ok(messages) => {
                println!("✅ 消息准备完成: {} 条", messages.len());

                match <ApiRequestNode as NodeBackend<InMemoryStorage>>::exec(
                    &mut self.regular_node,
                    messages,
                    &self.execution_context,
                )
                .await
                {
                    Ok(response) => {
                        result.duration = start_time.elapsed();
                        result.success = true;
                        result.response_length = response.len();

                        println!("✅ 响应成功!");
                        println!("📏 响应长度: {} 字符", response.len());
                        println!("⏱️  总耗时: {:.2?}", result.duration);

                        // 显示响应的前100个字符
                        let preview = if response.len() > 100 {
                            format!("{}...", &response[..100])
                        } else {
                            response
                        };
                        println!("📝 响应预览: {}", preview);
                    }
                    Err(e) => {
                        result.duration = start_time.elapsed();
                        result.error_message = Some(e.to_string());

                        println!("❌ 执行失败: {}", e);
                        println!("⏱️  总耗时: {:.2?}", result.duration);
                    }
                }
            }
            Err(e) => {
                result.duration = start_time.elapsed();
                result.error_message = Some(e.to_string());

                println!("❌ 消息准备失败: {}", e);
                println!("⏱️  总耗时: {:.2?}", result.duration);
            }
        }

        result
    }

    fn show_comparison(&self, streaming: ApiCallResult, regular: ApiCallResult) {
        println!("\n{}", "=".repeat(30) + " 对比结果 " + &"=".repeat(30));

        // 成功率对比
        println!("📊 **成功率对比**");
        println!(
            "   流式模式: {}",
            if streaming.success {
                "✅ 成功"
            } else {
                "❌ 失败"
            }
        );
        println!(
            "   非流式模式: {}",
            if regular.success {
                "✅ 成功"
            } else {
                "❌ 失败"
            }
        );

        // 性能对比
        if streaming.success && regular.success {
            println!("\n⏱️  **性能对比**");
            println!("   流式耗时:   {:.2?}", streaming.duration);
            println!("   非流式耗时: {:.2?}", regular.duration);

            let faster = if streaming.duration < regular.duration {
                format!("流式模式快 {:.2?}", regular.duration - streaming.duration)
            } else {
                format!("非流式模式快 {:.2?}", streaming.duration - regular.duration)
            };
            println!("   ⚡ {}", faster);

            println!("\n📏 **响应长度对比**");
            println!("   流式:   {} 字符", streaming.response_length);
            println!("   非流式: {} 字符", regular.response_length);
        }

        // 错误信息
        if !streaming.success {
            println!("\n❌ 流式模式错误: {:?}", streaming.error_message);
        }
        if !regular.success {
            println!("\n❌ 非流式模式错误: {:?}", regular.error_message);
        }

        println!("{}", "=".repeat(80));
    }

    fn show_summary(&self) {
        println!("\n🎯 **总结**");
        println!("================");
        println!("📋 **流式模式优势:**");
        println!("   • 实时响应，用户体验更好");
        println!("   • 适合长文本生成");
        println!("   • 可以提前中断响应");
        println!("   • 感知响应更快");
        println!();
        println!("📋 **非流式模式优势:**");
        println!("   • 实现更简单");
        println!("   • 适合短文本查询");
        println!("   • 批处理更高效");
        println!("   • 错误处理更直观");
        println!();
        println!("💡 **使用建议:**");
        println!("   • 交互式应用 → 选择流式模式");
        println!("   • 批量处理 → 选择非流式模式");
        println!("   • 实时聊天 → 选择流式模式");
        println!("   • API集成 → 根据需求选择");
    }
}

#[derive(Debug, Clone)]
struct ApiCallResult {
    mode: String,
    success: bool,
    duration: std::time::Duration,
    response_length: usize,
    error_message: Option<String>,
}

/// 性能基准测试
#[allow(dead_code)]
pub async fn benchmark_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏃‍♂️ 性能基准测试");

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo_key".to_string());

    let mut demo = ComparisonDemo::new(api_key).await?;

    let test_sizes = vec![
        ("短请求", "你好"),
        (
            "中等请求",
            "请解释一下什么是人工智能，包括其历史发展和主要应用领域",
        ),
        (
            "长请求",
            "请详细解释机器学习的各种算法类型，包括监督学习、无监督学习和强化学习，并为每种类型提供具体的算法示例和应用场景",
        ),
    ];

    for (name, query) in test_sizes {
        println!("\n🧪 测试: {}", name);
        println!("📝 查询: {}", query);

        demo.store.set("input".to_string(), json!(query))?;

        // 进行多次测试取平均值
        let mut streaming_times = Vec::new();
        let mut regular_times = Vec::new();

        for i in 0..3 {
            println!("\n第 {} 轮测试:", i + 1);

            let streaming_result = demo.test_streaming_node("test query").await;
            if streaming_result.success {
                streaming_times.push(streaming_result.duration);
            }

            let regular_result = demo.test_regular_node("test query").await;
            if regular_result.success {
                regular_times.push(regular_result.duration);
            }
        }

        if !streaming_times.is_empty() && !regular_times.is_empty() {
            let avg_streaming: std::time::Duration =
                streaming_times.iter().sum::<std::time::Duration>() / streaming_times.len() as u32;
            let avg_regular: std::time::Duration =
                regular_times.iter().sum::<std::time::Duration>() / regular_times.len() as u32;

            println!("\n📊 {} 平均耗时:", name);
            println!("   流式: {:.2?}", avg_streaming);
            println!("   非流式: {:.2?}", avg_regular);
        }
    }

    Ok(())
}
