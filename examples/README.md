# PocketFlow-rs 流式功能示例

本目录包含展示 PocketFlow-rs 流式API功能的示例应用。

## 📁 示例文件

### 1. 🤖 streaming_chat.rs
**完整功能的流式聊天应用**

- 支持实时流式对话
- 可以切换流式/非流式模式
- 保持对话历史
- 交互式命令界面
- 错误处理和超时管理

**运行方式:**
```bash
# 设置API密钥（必需）
export OPENAI_API_KEY=your_api_key_here

# 运行聊天应用
cargo run --example streaming_chat --features builtin-llm
```

**功能特性:**
- 输入 `stream on/off` 切换模式
- 输入 `clear` 清除历史
- 输入 `help` 查看帮助
- 输入 `quit` 退出

### 2. 🔄 simple_chatbot.rs
**简单的聊天机器人示例**

- 基础流式/非流式对话
- 单轮对话（不保持历史）
- 基本的命令控制
- 适合学习和理解基础概念

**运行方式:**
```bash
cargo run --example simple_chatbot --features builtin-llm
```

**功能特性:**
- 输入 `toggle` 切换模式
- 输入 `help` 查看帮助
- 输入 `bye` 退出

### 3. ⚖️ streaming_comparison.rs
**流式与非流式性能对比**

- 同时测试两种模式
- 性能基准测试
- 详细的对比分析
- 多个测试场景

**运行方式:**
```bash
cargo run --example streaming_comparison --features builtin-llm
```

**对比维度:**
- 响应时间
- 用户体验
- 资源使用
- 错误处理

### 4. 🚀 api_request_enhanced.rs
**API功能演示**

- 配置构建器模式演示
- 不同输入格式展示
- 实际应用场景示例
- 最佳实践建议

**运行方式:**
```bash
cargo run --example api_request_enhanced --features builtin-llm
```

## 🔧 环境设置

### 必需设置
```bash
# 设置OpenAI API密钥
export OPENAI_API_KEY=your_actual_api_key

# 或者创建 .env 文件
echo "OPENAI_API_KEY=your_actual_api_key" > .env
```

### 编译功能
```bash
# 包含LLM功能的编译
cargo build --features builtin-llm

# 运行特定示例
cargo run --example streaming_chat --features builtin-llm
```

## 📚 学习路径

### 1. 初学者
**推荐顺序:**
1. `api_request_enhanced.rs` - 了解基础概念
2. `simple_chatbot.rs` - 体验基本功能
3. `streaming_chat.rs` - 完整应用体验

### 2. 开发者
**推荐顺序:**
1. `streaming_comparison.rs` - 理解性能差异
2. `streaming_chat.rs` - 学习完整实现
3. 根据需求自定义开发

## 💡 使用场景

### 🔄 流式模式适合
- **实时聊天应用** - 提供逐字响应体验
- **内容创作工具** - 实时显示生成过程
- **代码助手** - 逐步解释代码
- **交互式应用** - 需要即时反馈

### 📦 非流式模式适合
- **批量处理** - 处理大量数据
- **API集成** - 后端服务调用
- **文档生成** - 完整内容输出
- **数据分析** - 结构化响应

## 🛠️ 自定义开发

### 基础配置
```rust
use pocketflow_rs::prelude::*;

// 流式配置
let config = ApiConfig::new("your-api-key")
    .with_model("gpt-3.5-turbo".to_string())
    .with_stream(true)  // 启用流式
    .with_max_tokens(1000);

// 创建节点
let node = ApiRequestNode::new("input", "output", Action::simple("next"))
    .with_config(config)
    .with_system_message("你是一个有帮助的助手");
```

### 错误处理
```rust
match node.exec(messages, &context).await {
    Ok(response) => {
        // 处理成功响应
        println!("响应: {}", response);
    }
    Err(e) => {
        // 处理错误
        eprintln!("错误: {}", e);
        
        // 检查特定错误类型
        if e.to_string().contains("auth") {
            eprintln!("请检查API密钥设置");
        }
    }
}
```

## 🔍 故障排除

### 常见问题

**1. API密钥错误**
```
错误: auth/authentication failed
解决: 检查 OPENAI_API_KEY 环境变量设置
```

**2. 编译失败**
```
错误: feature not enabled
解决: 添加 --features builtin-llm 编译选项
```

**3. 运行时错误**
```
错误: connection timeout
解决: 检查网络连接，增加timeout设置
```

### 调试技巧

**启用详细日志:**
```bash
RUST_LOG=debug cargo run --example streaming_chat --features builtin-llm
```

**测试网络连接:**
```bash
curl -H "Authorization: Bearer $OPENAI_API_KEY" https://api.openai.com/v1/models
```

## 📋 最佳实践

### 性能优化
- 合理设置 `max_tokens` 限制
- 使用适当的 `temperature` 值
- 流式模式用于交互，非流式用于批处理

### 错误处理
- 实现重试机制
- 设置合理的超时时间
- 优雅处理网络错误

### 用户体验
- 流式模式显示加载指示器
- 提供取消操作功能
- 实现响应缓存机制

## 🚀 下一步

1. **阅读源码** - 理解实现原理
2. **自定义配置** - 根据需求调整参数
3. **集成应用** - 将功能集成到你的项目
4. **性能调优** - 根据使用场景优化配置
5. **扩展功能** - 基于基础功能开发新特性

---

更多信息请参考：
- [STREAMING_IMPLEMENTATION.md](../STREAMING_IMPLEMENTATION.md) - 技术实现详情
- [README.md](../README.md) - 项目总体介绍
- [文档](../docs/) - 详细文档