use pocketflow_rs::Action;
#[cfg(feature = "builtin-llm")]
use pocketflow_rs::node::builtin::{ApiConfig, ApiRequestNode};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(feature = "builtin-llm"))]
    {
        println!("❌ 此示例需要启用 'builtin-llm' feature");
        println!("请使用以下命令运行：");
        println!("cargo run --example api_request_enhanced --features builtin-llm");
        return Ok(());
    }

    #[cfg(feature = "builtin-llm")]
    run_examples().await
}

#[cfg(feature = "builtin-llm")]
async fn run_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PocketFlow-rs ApiRequestNode 功能演示");
    println!("========================================");

    demo_config_patterns().await?;
    demo_streaming_features().await?;
    demo_different_inputs().await?;
    demo_real_world_usage().await?;

    Ok(())
}

/// 演示配置模式
async fn demo_config_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n✅ 配置构建器模式演示");
    println!("---------------------");

    // 基础配置
    let _basic_config = ApiConfig::new("test-api-key")
        .with_model("gpt-3.5-turbo".to_string())
        .with_max_tokens(100)
        .with_temperature(0.8);

    println!("📋 基础配置创建完成");

    // 流式配置
    let _streaming_config = ApiConfig::new("test-api-key")
        .with_model("gpt-4".to_string())
        .with_stream(true) // 启用流式
        .with_max_tokens(500)
        .with_temperature(0.7);

    println!("📋 流式配置创建完成");

    // 高级配置
    let _advanced_config = ApiConfig::new("test-api-key")
        .with_model("gpt-4".to_string())
        .with_stream(true)
        .with_max_tokens(1000)
        .with_temperature(0.6)
        .with_top_p(0.9)
        .with_frequency_penalty(0.1)
        .with_presence_penalty(0.1)
        .with_timeout(60);

    println!("📋 高级配置创建完成");

    Ok(())
}

/// 演示流式功能
async fn demo_streaming_features() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 流式功能演示");
    println!("----------------");

    // 流式节点
    let _streaming_node = ApiRequestNode::new("prompt", "response", Action::simple("end"))
        .with_config(
            ApiConfig::new("demo-key")
                .with_model("gpt-3.5-turbo".to_string())
                .with_stream(true) // 启用流式
                .with_max_tokens(200),
        );

    println!("✅ 创建流式API节点:");
    println!("   输入键: prompt");
    println!("   输出键: response");
    println!("   流式模式: 已启用");

    // 非流式节点
    let _regular_node = ApiRequestNode::new("messages", "output", Action::simple("done"))
        .with_config(
            ApiConfig::new("demo-key")
                .with_model("gpt-3.5-turbo".to_string())
                .with_stream(false) // 禁用流式
                .with_max_tokens(200),
        );

    println!("\n✅ 创建常规API节点:");
    println!("   输入键: messages");
    println!("   输出键: output");
    println!("   流式模式: 已禁用");

    Ok(())
}

/// 演示不同输入类型
async fn demo_different_inputs() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 不同输入类型演示");
    println!("-------------------");

    // 包含系统消息的节点
    let _node_with_system = ApiRequestNode::new("input", "output", Action::simple("end"))
        .with_config(
            ApiConfig::new("demo-key")
                .with_model("gpt-3.5-turbo".to_string())
                .with_stream(true),
        )
        .with_system_message("你是一个有帮助的AI助手，请用中文回答问题。");

    println!("✅ 带系统消息的节点:");
    println!("   系统消息: 已设置");
    println!("   用途: 定义AI助手的行为和回答风格");

    // 支持重试的节点
    let _node_with_retries = ApiRequestNode::new("input", "output", Action::simple("end"))
        .with_config(
            ApiConfig::new("demo-key")
                .with_model("gpt-3.5-turbo".to_string())
                .with_stream(false)
                .with_timeout(30),
        )
        .with_retries(3);

    println!("\n✅ 支持重试的节点:");
    println!("   最大重试: 3次");
    println!("   超时: 30秒");
    println!("   用途: 提高API调用的可靠性");

    // 演示输入格式
    println!("\n📋 支持的输入格式:");

    // 1. 简单字符串
    let simple_input = json!("你好，请介绍一下你自己");
    println!("   1. 简单字符串: {}", simple_input);

    // 2. 消息数组
    let message_input = json!([
        {
            "role": "user",
            "content": "什么是人工智能？"
        }
    ]);
    println!(
        "   2. 消息数组: {}",
        serde_json::to_string_pretty(&message_input)?
    );

    // 3. 多轮对话
    let conversation_input = json!([
        {
            "role": "user",
            "content": "我想学编程"
        },
        {
            "role": "assistant",
            "content": "编程是一项很有用的技能！你想学习什么编程语言？"
        },
        {
            "role": "user",
            "content": "推荐一个适合初学者的"
        }
    ]);
    println!(
        "   3. 多轮对话: {}",
        serde_json::to_string_pretty(&conversation_input)?
    );

    Ok(())
}

/// 演示实际使用场景
async fn demo_real_world_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌍 实际应用场景演示");
    println!("-------------------");

    // 场景1：实时聊天机器人
    println!("🤖 场景1: 实时聊天机器人");
    let _chatbot_node = ApiRequestNode::new(
        "user_input",
        "bot_response",
        Action::simple("chat_continue"),
    )
    .with_config(
        ApiConfig::new("your-api-key")
            .with_model("gpt-3.5-turbo".to_string())
            .with_stream(true) // 流式响应提供实时用户体验
            .with_max_tokens(300)
            .with_temperature(0.7),
    )
    .with_system_message("你是一个友善的聊天机器人，用轻松自然的语调与用户对话。");

    println!("   ✅ 配置: 流式模式，实时响应");
    println!("   ✅ 适用: 在线客服，个人助手");

    // 场景2：内容生成系统
    println!("\n📝 场景2: 内容生成系统");
    let _content_generator =
        ApiRequestNode::new("topic", "content", Action::simple("content_ready"))
            .with_config(
                ApiConfig::new("your-api-key")
                    .with_model("gpt-4".to_string())
                    .with_stream(false) // 批处理模式，一次性生成完整内容
                    .with_max_tokens(2000)
                    .with_temperature(0.8),
            )
            .with_system_message("你是一个专业的内容创作者，擅长写作各种类型的文章。");

    println!("   ✅ 配置: 非流式模式，完整输出");
    println!("   ✅ 适用: 博客文章，营销文案");

    // 场景3：代码辅助工具
    println!("\n💻 场景3: 代码辅助工具");
    let _code_assistant =
        ApiRequestNode::new("code_question", "code_answer", Action::simple("code_help"))
            .with_config(
                ApiConfig::new("your-api-key")
                    .with_model("gpt-4".to_string())
                    .with_stream(true) // 流式显示代码解释过程
                    .with_max_tokens(1000)
                    .with_temperature(0.3), // 较低温度保证代码准确性
            )
            .with_system_message("你是一个专业的编程助手，提供准确的代码解释和解决方案。");

    println!("   ✅ 配置: 流式模式，逐步解释");
    println!("   ✅ 适用: IDE插件，编程学习");

    println!("\n💡 选择建议:");
    println!("   • 交互性强的应用 → 使用流式模式");
    println!("   • 批量处理任务 → 使用非流式模式");
    println!("   • 实时反馈需求 → 使用流式模式");
    println!("   • 完整内容生成 → 使用非流式模式");

    Ok(())
}
