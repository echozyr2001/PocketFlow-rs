use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
};
use futures::StreamExt;
use pocketflow_rs::{
    ExecutionContext, InMemoryStorage, SharedStore, node::builtin::llm::ApiConfig,
};
use serde_json::json;
use std::io::{self, Write};
use std::time::Duration;

/// 带打字机效果的流式聊天应用示例
/// 展示如何实现实时的打字机效果响应
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⌨️  PocketFlow-rs 打字机效果聊天应用");
    println!("========================================");
    println!("演示AI响应的实时打字机效果");
    println!("输入 'quit' 退出，输入 'clear' 清除历史");
    println!();

    // 初始化应用
    let mut chat_app = TypewriterChatApp::new().await?;

    // 开始聊天循环
    chat_app.run_chat_loop().await?;

    Ok(())
}

/// 打字机效果聊天应用
pub struct TypewriterChatApp {
    store: SharedStore<InMemoryStorage>,
    execution_context: ExecutionContext,
    conversation_history: Vec<serde_json::Value>,
    client: Client<OpenAIConfig>,
    config: ApiConfig,
}

impl TypewriterChatApp {
    /// 创建新的聊天应用
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let store: SharedStore<InMemoryStorage> = SharedStore::new();
        let execution_context = ExecutionContext::new(3, Duration::from_secs(30));

        // 检查API密钥
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "demo_key".to_string());

        if api_key == "demo_key" {
            println!("⚠️  未设置 OPENAI_API_KEY 环境变量，将使用演示模式");
            println!("   实际API调用将失败，但可以查看打字机效果演示");
            println!();
        }

        // 创建API配置
        let config = ApiConfig {
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
            stream: true, // 强制启用流式响应
        };

        // 创建OpenAI客户端
        let mut client_config = OpenAIConfig::new().with_api_key(&config.api_key);
        if let Some(ref base_url) = config.base_url {
            client_config = client_config.with_api_base(base_url);
        }
        let client = Client::with_config(client_config);

        Ok(Self {
            store,
            execution_context,
            conversation_history: Vec::new(),
            client,
            config,
        })
    }

    /// 运行聊天循环
    pub async fn run_chat_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // 获取用户输入
            print!("👤 你: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let user_input = input.trim();

            // 处理特殊命令
            match user_input {
                "quit" | "exit" => {
                    println!("👋 再见！");
                    break;
                }
                "clear" => {
                    self.conversation_history.clear();
                    println!("🗑️  对话历史已清除");
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
            if let Err(e) = self.process_chat_with_typewriter_effect(user_input).await {
                println!("❌ 处理消息时出错: {}", e);
            }

            println!(); // 添加空行分隔
        }

        Ok(())
    }

    /// 使用打字机效果处理聊天消息
    async fn process_chat_with_typewriter_effect(
        &mut self,
        user_input: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 添加用户消息到历史
        let user_message = json!({
            "role": "user",
            "content": user_input
        });
        self.conversation_history.push(user_message);

        // 构建消息请求
        let mut messages = Vec::new();

        // 添加系统提示
        messages.push(ChatCompletionRequestMessage::System(
            async_openai::types::ChatCompletionRequestSystemMessage {
                content: "你是一个友好、有帮助的AI助手。请用简洁且有趣的方式回答问题。".into(),
                name: None,
            },
        ));

        // 添加对话历史
        for msg in &self.conversation_history {
            let role = msg["role"].as_str().unwrap_or("user");
            let content = msg["content"].as_str().unwrap_or("");

            match role {
                "user" => {
                    messages.push(ChatCompletionRequestMessage::User(
                        async_openai::types::ChatCompletionRequestUserMessage {
                            content: content.into(),
                            name: None,
                        },
                    ));
                }
                "assistant" => {
                    messages.push(ChatCompletionRequestMessage::Assistant(
                        async_openai::types::ChatCompletionRequestAssistantMessage {
                            content: Some(content.into()),
                            name: None,
                            ..Default::default()
                        },
                    ));
                }
                _ => {} // 忽略不支持的角色
            }
        }

        // 创建请求
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages(messages)
            .max_tokens(self.config.max_tokens.unwrap_or(1000))
            .temperature(self.config.temperature.unwrap_or(0.7))
            .stream(true)
            .build()?;

        // 显示AI响应开始
        print!("🤖 AI助手: ");
        io::stdout().flush()?;

        let start_time = std::time::Instant::now();

        // 检查是否为演示模式
        if self.config.api_key == "demo_key" {
            self.simulate_typewriter_effect().await?;
            return Ok(());
        }

        // 发送流式请求并实现打字机效果
        match self.client.chat().create_stream(request).await {
            Ok(mut stream) => {
                let mut accumulated_response = String::new();

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(response) => {
                            // 提取增量内容
                            if let Some(choice) = response.choices.first() {
                                if let Some(delta) = &choice.delta.content {
                                    // 打字机效果：逐字符显示
                                    for ch in delta.chars() {
                                        print!("{}", ch);
                                        io::stdout().flush()?;
                                        accumulated_response.push(ch);

                                        // 添加打字延迟效果（可调整）
                                        if ch != ' ' {
                                            // 空格不延迟
                                            tokio::time::sleep(Duration::from_millis(30)).await;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("\n❌ 流处理错误: {}", e);
                            break;
                        }
                    }
                }

                println!(); // 换行
                let duration = start_time.elapsed();
                println!(
                    "⏱️  响应时间: {:.2?} | 字符数: {}",
                    duration,
                    accumulated_response.len()
                );

                // 添加AI响应到历史
                if !accumulated_response.is_empty() {
                    let assistant_message = json!({
                        "role": "assistant",
                        "content": accumulated_response
                    });
                    self.conversation_history.push(assistant_message);
                }

                // 限制历史长度
                if self.conversation_history.len() > 20 {
                    self.conversation_history.drain(0..2);
                }
            }
            Err(e) => {
                println!("❌ API调用失败: {}", e);
                let duration = start_time.elapsed();

                if e.to_string().contains("auth") {
                    println!("💡 提示: 请设置有效的 OPENAI_API_KEY 环境变量");
                    println!("   export OPENAI_API_KEY=your_api_key_here");
                }

                println!("⏱️  失败时间: {:.2?}", duration);
            }
        }

        Ok(())
    }

    /// 模拟打字机效果（演示模式）
    async fn simulate_typewriter_effect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let demo_responses = [
            "这是一个打字机效果的演示。",
            "实际使用需要设置有效的 API 密钥。",
            "每个字符都会以打字机的速度显示出来，创造更自然的对话体验。",
            "您可以尝试问我任何问题！",
        ];

        let response = demo_responses[self.conversation_history.len() % demo_responses.len()];

        // 模拟打字机效果
        for ch in response.chars() {
            print!("{}", ch);
            io::stdout().flush()?;

            // 根据字符类型调整延迟
            let delay_ms = match ch {
                '，' | '。' | '！' | '？' => 200, // 标点符号较长停顿
                ' ' => 50,                        // 空格短停顿
                _ => 80,                          // 普通字符中等停顿
            };

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        println!(); // 换行
        println!("⏱️  演示模式 | 字符数: {}", response.len());

        // 添加到历史
        let assistant_message = json!({
            "role": "assistant",
            "content": response
        });
        self.conversation_history.push(assistant_message);

        Ok(())
    }

    /// 显示帮助信息
    fn show_help(&self) {
        println!();
        println!("📖 可用命令:");
        println!("  help        - 显示此帮助信息");
        println!("  clear       - 清除对话历史");
        println!("  quit/exit   - 退出应用");
        println!();
        println!("✨ 打字机效果特性:");
        println!("  - 字符逐个显示，模拟真实打字");
        println!("  - 标点符号有较长停顿");
        println!("  - 空格有短暂停顿");
        println!("  - 创造自然的对话节奏");
        println!();
        println!("💡 提示:");
        println!("  - 需要设置 OPENAI_API_KEY 环境变量");
        println!("  - 演示模式下会显示预设的打字机效果");
        println!("  - 对话历史自动保持上下文");
        println!();
    }
}

/// 演示打字机效果的配置选项
#[allow(dead_code)]
pub struct TypewriterConfig {
    /// 普通字符的延迟（毫秒）
    pub char_delay: u64,
    /// 空格的延迟（毫秒）
    pub space_delay: u64,
    /// 标点符号的延迟（毫秒）
    pub punctuation_delay: u64,
    /// 是否启用打字机效果
    pub enabled: bool,
}

impl Default for TypewriterConfig {
    fn default() -> Self {
        Self {
            char_delay: 30,
            space_delay: 10,
            punctuation_delay: 150,
            enabled: true,
        }
    }
}

/// 高级打字机效果实现（可扩展功能）
#[allow(dead_code)]
impl TypewriterChatApp {
    /// 使用自定义配置的打字机效果
    async fn advanced_typewriter_effect(
        &self,
        text: &str,
        config: &TypewriterConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !config.enabled {
            print!("{}", text);
            io::stdout().flush()?;
            return Ok(());
        }

        for ch in text.chars() {
            print!("{}", ch);
            io::stdout().flush()?;

            let delay_ms = match ch {
                '，' | '。' | '！' | '？' | '；' | '：' => config.punctuation_delay,
                ' ' | '\t' => config.space_delay,
                '\n' => 0, // 换行不延迟
                _ => config.char_delay,
            };

            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        Ok(())
    }
}
