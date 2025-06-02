#[cfg(feature = "builtin-llm")]
use pocketflow_rs::node::builtin::{ApiConfig, ApiRequestNode};
use pocketflow_rs::{Action, ExecutionContext, InMemoryStorage, SharedStore, node::NodeBackend};
use serde_json::json;
use std::io::{self, Write};
use std::time::Duration;
use tokio;

/// 简单的流式聊天机器人
/// 演示 PocketFlow-rs 流式API功能的基本用法
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "builtin-llm"))]
    {
        println!("❌ 此示例需要启用 'builtin-llm' feature");
        println!("请使用以下命令运行：");
        println!("cargo run --example simple_chatbot --features builtin-llm");
        return Ok(());
    }

    #[cfg(feature = "builtin-llm")]
    run_chatbot().await
}

#[cfg(feature = "builtin-llm")]
async fn run_chatbot() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 简单流式聊天机器人");
    println!("===================");
    println!("提示: 需要设置 OPENAI_API_KEY 环境变量");
    println!("输入 'bye' 退出聊天\n");

    // 检查API密钥
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  未找到 OPENAI_API_KEY，使用演示密钥");
        "demo_key".to_string()
    });

    // 初始化存储和执行上下文
    let mut store: SharedStore<InMemoryStorage> = SharedStore::new();
    let execution_context = ExecutionContext::new(3, Duration::from_secs(30));

    // 创建流式API配置
    let streaming_config = ApiConfig {
        api_key: api_key.clone(),
        base_url: None,
        org_id: None,
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.7),
        top_p: None,
        frequency_penalty: None,
        presence_penalty: None,
        timeout: Some(30),
        stream: true, // 启用流式
    };

    // 创建非流式API配置用于对比
    let regular_config = ApiConfig {
        stream: false, // 禁用流式
        ..streaming_config.clone()
    };

    // 创建API节点
    let mut streaming_node = ApiRequestNode::new("input", "output", Action::simple("next"))
        .with_config(streaming_config)
        .with_system_message("你是一个友善的AI助手，用中文回答问题。");

    let mut regular_node = ApiRequestNode::new("input", "output", Action::simple("next"))
        .with_config(regular_config)
        .with_system_message("你是一个友善的AI助手，用中文回答问题。");

    // 聊天循环
    let mut use_streaming = true;

    loop {
        // 显示提示符
        let mode = if use_streaming { "流式" } else { "常规" };
        print!("\n[{}] 你: ", mode);
        io::stdout().flush()?;

        // 读取用户输入
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let user_input = input.trim();

        // 处理退出命令
        if user_input == "bye" || user_input == "quit" {
            println!("👋 再见！");
            break;
        }

        // 切换模式命令
        if user_input == "toggle" {
            use_streaming = !use_streaming;
            println!(
                "✅ 已切换到{}模式",
                if use_streaming { "流式" } else { "常规" }
            );
            continue;
        }

        // 显示帮助
        if user_input == "help" {
            println!("💡 可用命令:");
            println!("  toggle - 切换流式/常规模式");
            println!("  help   - 显示帮助");
            println!("  bye    - 退出");
            continue;
        }

        if user_input.is_empty() {
            continue;
        }

        // 处理聊天消息
        let start_time = std::time::Instant::now();

        // 使用简单的字符串输入（单次对话，不保持历史）
        store.set("input".to_string(), json!(user_input))?;

        // 选择节点
        let node = if use_streaming {
            &mut streaming_node
        } else {
            &mut regular_node
        };

        print!("🤖 AI: ");
        io::stdout().flush()?;

        if use_streaming {
            println!("(流式响应...)");
        }

        // 执行API调用
        match node.prep(&store, &execution_context).await {
            Ok(messages) => {
                match <ApiRequestNode as NodeBackend<InMemoryStorage>>::exec(
                    node,
                    messages,
                    &execution_context,
                )
                .await
                {
                    Ok(response) => {
                        let duration = start_time.elapsed();

                        if use_streaming {
                            println!("{}", response);
                        } else {
                            println!("\r🤖 AI: {}", response);
                        }

                        println!("⏱️ 用时: {:.1?}", duration);
                    }
                    Err(e) => {
                        println!("❌ 错误: {}", e);
                        if e.to_string().contains("demo_key") {
                            println!("💡 请设置真实的API密钥: export OPENAI_API_KEY=your_key");
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ 准备失败: {}", e);
            }
        }
    }

    Ok(())
}

/// 演示不同输入格式的处理
#[allow(dead_code)]
async fn demo_input_formats() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 演示不同输入格式");

    let mut store: SharedStore<InMemoryStorage> = SharedStore::new();
    let execution_context = ExecutionContext::new(3, Duration::from_secs(10));

    let config = ApiConfig::default()
        .with_model("gpt-3.5-turbo".to_string())
        .with_stream(true);

    let mut node =
        ApiRequestNode::new("input", "output", Action::simple("next")).with_config(config);

    // 1. 简单字符串输入
    println!("\n1️⃣ 简单字符串输入:");
    store.set("input".to_string(), json!("你好"))?;
    demo_api_call(&mut node, &store, &execution_context).await;

    // 2. 消息数组输入
    println!("\n2️⃣ 消息数组输入:");
    store.set(
        "input".to_string(),
        json!([
            {"role": "user", "content": "什么是AI？"}
        ]),
    )?;
    demo_api_call(&mut node, &store, &execution_context).await;

    // 3. 带历史的对话
    println!("\n3️⃣ 带历史的对话:");
    store.set(
        "input".to_string(),
        json!([
            {"role": "user", "content": "我想学编程"},
            {"role": "assistant", "content": "编程是一项很有用的技能！你想学什么语言？"},
            {"role": "user", "content": "推荐一个适合初学者的语言"}
        ]),
    )?;
    demo_api_call(&mut node, &store, &execution_context).await;

    Ok(())
}

async fn demo_api_call(
    node: &mut ApiRequestNode,
    store: &SharedStore<InMemoryStorage>,
    context: &ExecutionContext,
) {
    match node.prep(store, context).await {
        Ok(messages) => {
            println!("✅ 准备了 {} 条消息", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                println!("   消息 {}: {:?}", i + 1, msg);
            }
        }
        Err(e) => println!("❌ 准备失败: {}", e),
    }
}
