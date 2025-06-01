use pocketflow_rs::{
    Action, ExecutionContext, InMemoryStorage, SharedStore,
    node::NodeBackend,
    node::builtin::llm::{ApiConfig, ApiRequestNode},
};
use serde_json::json;
use std::io::{self, Write};
use std::time::Duration;
use tokio;

/// 流式聊天应用示例
/// 展示如何使用 PocketFlow-rs 的流式 API 功能构建实时聊天体验
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 PocketFlow-rs 流式聊天应用");
    println!("=====================================");
    println!("说明：这是一个演示程序，需要设置 OPENAI_API_KEY 环境变量才能正常工作");
    println!("输入 'quit' 退出，输入 'stream on/off' 切换流式模式");
    println!();

    // 初始化应用
    let mut chat_app = ChatApplication::new().await?;

    // 开始聊天循环
    chat_app.run_chat_loop().await?;

    Ok(())
}

/// 聊天应用结构
pub struct ChatApplication {
    store: SharedStore<InMemoryStorage>,
    execution_context: ExecutionContext,
    streaming_node: ApiRequestNode,
    regular_node: ApiRequestNode,
    conversation_history: Vec<serde_json::Value>,
    use_streaming: bool,
}

impl ChatApplication {
    /// 创建新的聊天应用
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut store: SharedStore<InMemoryStorage> = SharedStore::new();
        let execution_context = ExecutionContext::new(3, Duration::from_secs(30));

        // 检查API密钥
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo_key".to_string());

        if api_key == "demo_key" {
            println!("⚠️  未设置 OPENAI_API_KEY 环境变量，将使用演示模式");
            println!("   实际API调用将失败，但可以查看流程演示");
            println!();
        }

        // 创建流式API配置
        let streaming_config = ApiConfig {
            api_key: api_key.clone(),
            base_url: None,
            org_id: None,
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            timeout: Some(30),
            stream: true, // 启用流式响应
        };

        // 创建常规API配置
        let regular_config = ApiConfig {
            stream: false, // 禁用流式响应
            ..streaming_config.clone()
        };

        // 创建API节点
        let streaming_node =
            ApiRequestNode::new("messages", "response", Action::simple("continue"))
                .with_config(streaming_config)
                .with_system_message("你是一个有帮助的AI助手。请友好、准确地回答用户的问题。");

        let regular_node = ApiRequestNode::new("messages", "response", Action::simple("continue"))
            .with_config(regular_config)
            .with_system_message("你是一个有帮助的AI助手。请友好、准确地回答用户的问题。");

        Ok(Self {
            store,
            execution_context,
            streaming_node,
            regular_node,
            conversation_history: Vec::new(),
            use_streaming: true, // 默认启用流式模式
        })
    }

    /// 运行聊天循环
    pub async fn run_chat_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // 显示当前模式
            let mode_indicator = if self.use_streaming {
                "🔄 流式"
            } else {
                "📦 常规"
            };
            print!("{} 模式 > ", mode_indicator);
            io::stdout().flush()?;

            // 读取用户输入
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let user_input = input.trim();

            // 处理特殊命令
            match user_input {
                "quit" | "exit" => {
                    println!("👋 再见！");
                    break;
                }
                "stream on" => {
                    self.use_streaming = true;
                    println!("✅ 已切换到流式模式");
                    continue;
                }
                "stream off" => {
                    self.use_streaming = false;
                    println!("✅ 已切换到常规模式");
                    continue;
                }
                "clear" => {
                    self.conversation_history.clear();
                    println!("🗑️ 对话历史已清除");
                    continue;
                }
                "help" => {
                    self.show_help();
                    continue;
                }
                _ if user_input.is_empty() => continue,
                _ => {}
            }

            // 处理聊天消息
            if let Err(e) = self.process_chat_message(user_input).await {
                println!("❌ 处理消息时出错: {}", e);
            }

            println!(); // 添加空行分隔
        }

        Ok(())
    }

    /// 处理聊天消息
    async fn process_chat_message(
        &mut self,
        user_input: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 添加用户消息到历史
        let user_message = json!({
            "role": "user",
            "content": user_input
        });
        self.conversation_history.push(user_message);

        // 准备消息数组
        let messages = json!(self.conversation_history);
        self.store.set("messages".to_string(), messages)?;

        // 选择使用的节点
        let node = if self.use_streaming {
            &mut self.streaming_node
        } else {
            &mut self.regular_node
        };

        println!();
        print!("🤖 AI助手: ");
        io::stdout().flush()?;

        let start_time = std::time::Instant::now();

        // 执行API调用
        match node.prep(&self.store, &self.execution_context).await {
            Ok(prepared_messages) => {
                // 显示正在处理的消息数量
                if self.use_streaming {
                    println!("(流式响应中...)");
                } else {
                    print!("(处理中...)");
                    io::stdout().flush()?;
                }

                // 执行API请求
                match <ApiRequestNode as NodeBackend<InMemoryStorage>>::exec(
                    node,
                    prepared_messages,
                    &self.execution_context,
                )
                .await
                {
                    Ok(response) => {
                        let duration = start_time.elapsed();

                        if !self.use_streaming {
                            // 非流式模式：一次性显示完整响应
                            println!("\r🤖 AI助手: {}", response);
                        } else {
                            // 流式模式：响应已经在流式处理中显示
                            println!("{}", response);
                        }

                        println!("⏱️  响应时间: {:.2?}", duration);

                        // 添加AI响应到历史
                        let assistant_message = json!({
                            "role": "assistant",
                            "content": response
                        });
                        self.conversation_history.push(assistant_message);

                        // 限制历史长度（保留最近20条消息）
                        if self.conversation_history.len() > 20 {
                            self.conversation_history.drain(0..2); // 移除最早的一轮对话
                        }
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        println!("❌ API调用失败: {}", e);

                        if e.to_string().contains("demo_key") || e.to_string().contains("auth") {
                            println!("💡 提示: 请设置有效的 OPENAI_API_KEY 环境变量");
                            println!("   export OPENAI_API_KEY=your_api_key_here");
                        }

                        println!("⏱️  失败时间: {:.2?}", duration);
                    }
                }
            }
            Err(e) => {
                println!("❌ 消息准备失败: {}", e);
            }
        }

        Ok(())
    }

    /// 显示帮助信息
    fn show_help(&self) {
        println!();
        println!("📖 可用命令:");
        println!("  help        - 显示此帮助信息");
        println!("  stream on   - 启用流式模式（实时显示响应）");
        println!("  stream off  - 启用常规模式（完整响应一次显示）");
        println!("  clear       - 清除对话历史");
        println!("  quit/exit   - 退出应用");
        println!();
        println!("💡 提示:");
        println!("  - 流式模式提供实时响应体验，适合长对话");
        println!("  - 常规模式等待完整响应后显示，适合短查询");
        println!("  - 对话历史自动保持，支持上下文对话");
        println!();
    }
}

/// 演示批量测试功能
#[allow(dead_code)]
async fn demo_batch_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 演示批量处理功能");

    let mut store: SharedStore<InMemoryStorage> = SharedStore::new();
    let execution_context = ExecutionContext::new(3, Duration::from_secs(30));

    // 创建测试配置
    let config = ApiConfig {
        api_key: "demo_key".to_string(),
        base_url: None,
        org_id: None,
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        timeout: Some(10),
        stream: true,
    };

    let mut node =
        ApiRequestNode::new("input", "output", Action::simple("next")).with_config(config);

    // 测试不同类型的消息
    let test_messages = vec![
        json!("你好，请简单介绍一下人工智能"),
        json!([
            {"role": "user", "content": "什么是机器学习？"},
            {"role": "assistant", "content": "机器学习是人工智能的一个重要分支..."},
            {"role": "user", "content": "请举个实际应用的例子"}
        ]),
        json!("解释一下量子计算的基本概念"),
    ];

    for (i, test_input) in test_messages.into_iter().enumerate() {
        println!(
            "\n📝 测试 {} - 输入类型: {}",
            i + 1,
            if test_input.is_string() {
                "简单文本"
            } else {
                "对话历史"
            }
        );

        store.set("input".to_string(), test_input)?;

        match node.prep(&store, &execution_context).await {
            Ok(messages) => {
                println!("✅ 消息准备成功: {} 条消息", messages.len());
                // 注意: 这里会因为没有有效API密钥而失败，但展示了流程
            }
            Err(e) => {
                println!("❌ 消息准备失败: {}", e);
            }
        }
    }

    Ok(())
}

/// 演示配置比较
#[allow(dead_code)]
fn demo_config_comparison() {
    println!("⚙️  配置对比演示");

    // 流式配置
    let streaming_config = ApiConfig::default()
        .with_model("gpt-4".to_string())
        .with_stream(true)
        .with_max_tokens(2000)
        .with_temperature(0.8);

    // 常规配置
    let regular_config = ApiConfig::default()
        .with_model("gpt-3.5-turbo".to_string())
        .with_stream(false)
        .with_max_tokens(1000)
        .with_temperature(0.7);

    println!("🔄 流式配置:");
    println!("  模型: {}", streaming_config.model);
    println!("  流式: {}", streaming_config.stream);
    println!("  最大令牌: {:?}", streaming_config.max_tokens);
    println!("  温度: {:?}", streaming_config.temperature);

    println!("\n📦 常规配置:");
    println!("  模型: {}", regular_config.model);
    println!("  流式: {}", regular_config.stream);
    println!("  最大令牌: {:?}", regular_config.max_tokens);
    println!("  温度: {:?}", regular_config.temperature);
}
