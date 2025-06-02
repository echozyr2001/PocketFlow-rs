# PocketFlow-rs Examples

*Comprehensive examples showcasing PocketFlow-rs capabilities from basic concepts to advanced AI workflows*

<div align="center">

|  Example  | Difficulty | Description | 
| :--------: | :--------: | :---------- |
| **🚀 Basics** | | |
| [Hello World](#hello-world) | ☆☆☆ *Starter* | Your first PocketFlow-rs workflow |
| [Storage Systems](#storage-systems) | ☆☆☆ *Starter* | In-memory, file, Redis, and database storage |
| [Node Types](#node-types) | ☆☆☆ *Starter* | Built-in nodes and custom node creation |
| **🔄 Flow Control** | | |
| [Flow Builder](#flow-builder) | ★☆☆ *Basic* | Building workflows with the fluent API |
| [Conditional Routes](#conditional-routes) | ★☆☆ *Basic* | Dynamic flow control with conditions |
| [Action System](#action-system) | ★☆☆ *Basic* | Simple, parameterized, and conditional actions |
| **🤖 AI Integration** | | |
| [LLM Chat](#llm-chat) | ★☆☆ *Basic* | Simple AI chat integration |
| [Streaming Response](#streaming-response) | ★★☆ *Intermediate* | Real-time streaming AI responses |
| [Chat with Memory](#chat-with-memory) | ★★☆ *Intermediate* | Persistent conversation history |
| **⚙️ Advanced Patterns** | | |
| [Workflow Orchestration](#workflow-orchestration) | ★★☆ *Intermediate* | Complex multi-step workflows |
| [RAG System](#rag-system) | ★★★ *Advanced* | Retrieval-augmented generation |
| [Agent Framework](#agent-framework) | ★★★ *Advanced* | Autonomous AI agent with tools |
| **🌌 CosmoAI Integration** | | |
| [Code Generation](#code-generation) | ★★★ *Advanced* | AI-powered code generation |
| [Meta-Flow](#meta-flow) | ★★★ *Expert* | Self-modifying workflows |
| [Bootstrap Demo](#bootstrap-demo) | ★★★ *Expert* | CosmoAI self-improvement showcase |

</div>

## 🎯 Getting Started

### Prerequisites
```bash
# Set up environment
export OPENAI_API_KEY=your_api_key_here  # Required for AI examples

# Install dependencies
cargo build
```

### Quick Start
```bash
# Run the hello world example
cargo run --example hello_world

# Try an AI-powered example (requires API key)
cargo run --example llm_chat --features "builtin-llm"

# Explore storage backends
cargo run --example storage_showcase --features "storage-redis,storage-database"
```

---

## 🚀 Basic Examples

### Hello World
**File:** `examples/01_hello_world.rs`

Your first PocketFlow-rs workflow - demonstrates the three-phase execution model.

```rust
// Minimal working example
let node = LogNode::new("Hello, PocketFlow-rs!", Action::simple("done"));
let mut flow = FlowBuilder::new()
    .start_node("start")
    .terminal_action("done")
    .node("start", node)
    .build();

let mut store = SharedStore::new();
flow.execute(&mut store).await?;
```

**Key Concepts:**
- Three-phase execution: prep → exec → post
- Shared store for data communication
- Actions for flow control

### Storage Systems
**File:** `examples/02_storage_showcase.rs`

Comprehensive tour of all storage backends with performance comparisons.

**Features Covered:**
- In-memory storage (default)
- File-based persistence
- Redis distributed storage  
- PostgreSQL/MySQL database storage
- Performance and use case comparisons

### Node Types
**File:** `examples/03_node_showcase.rs`

Complete overview of built-in nodes and custom node creation.

**Built-in Nodes:**
- `LogNode` - Debug output
- `SetValueNode` - Store data
- `GetValueNode` - Retrieve data
- `DelayNode` - Time-based control
- `FunctionNode` - Custom logic
- `ConditionalNode` - Branch execution

---

## 🔄 Flow Control Examples

### Flow Builder
**File:** `examples/04_flow_builder.rs`

Master the fluent API for building complex workflows.

```rust
FlowBuilder::new()
    .start_node("input")
    .terminal_action("complete")
    .max_steps(100)
    .node("input", input_node)
    .node("process", process_node) 
    .node("output", output_node)
    .route("input", "continue", "process")
    .route("process", "success", "output")
    .route("process", "retry", "input")
    .build()
```

### Conditional Routes
**File:** `examples/05_conditional_routes.rs`

Dynamic flow control based on runtime conditions.

**Condition Types:**
- Key existence checks
- Value equality/comparison
- Custom condition functions
- Complex boolean logic

### Action System
**File:** `examples/06_action_system.rs`

Complete guide to the action system for flow transitions.

**Action Types:**
- Simple actions (`"continue"`)
- Parameterized actions (with data)
- Conditional actions (with guards)
- Composite actions (multiple outcomes)

---

## 🤖 AI Integration Examples

### LLM Chat
**File:** `examples/07_llm_chat.rs`

Basic AI chat integration with OpenAI API.

**Requirements:** `--features "builtin-llm"`

```rust
let chat_node = ApiRequestNode::new("input", "response", Action::simple("done"))
    .with_model("gpt-4")
    .with_temperature(0.7)
    .with_system_message("You are a helpful assistant");
```

### Streaming Response
**File:** `examples/08_streaming_response.rs`

Real-time streaming responses for better user experience.

**Features:**
- Token-by-token streaming
- Cancellation support
- Progress indicators
- Error handling

### Chat with Memory
**File:** `examples/09_chat_memory.rs`

Persistent conversation history with vector embeddings.

**Technologies:**
- Conversation history management
- Vector embeddings for context
- Similarity search
- Memory optimization

---

## ⚙️ Advanced Pattern Examples

### Workflow Orchestration
**File:** `examples/10_workflow_orchestration.rs`

Complex multi-step workflow with error handling and retries.

**Pattern Features:**
- Multi-stage processing pipeline
- Error recovery strategies
- Parallel execution branches
- Result aggregation

### RAG System
**File:** `examples/11_rag_system.rs`

Complete Retrieval-Augmented Generation implementation.

**Components:**
- Document ingestion and chunking
- Vector database integration
- Semantic search
- Context-aware generation

### Agent Framework
**File:** `examples/12_agent_framework.rs`

Autonomous AI agent with tool integration.

**Capabilities:**
- Tool selection and usage
- Multi-step reasoning
- External API integration
- State management

---

## 🌌 CosmoAI Integration Examples

### Code Generation
**File:** `examples/13_code_generation.rs`

AI-powered code generation optimized for PocketFlow-rs patterns.

**Features:**
- Template-based generation
- Type-safe generation
- Integration testing
- Code validation

### Meta-Flow
**File:** `examples/14_meta_flow.rs`

Self-modifying workflows that can update their own structure.

**Concepts:**
- Dynamic node creation
- Runtime flow modification
- Self-reflection capabilities
- Adaptive behavior

### Bootstrap Demo
**File:** `examples/15_bootstrap_demo.rs`

Showcase of CosmoAI's self-improvement capabilities.

**Vision:**
- CosmoAI uses PocketFlow-rs to improve itself
- Recursive self-enhancement
- Automated testing and validation
- Continuous improvement loop

---

## 📚 Learning Path

### 🌱 Beginner (1-3 weeks)
1. **Week 1:** Hello World → Storage → Nodes
2. **Week 2:** Flow Builder → Conditional Routes → Actions  
3. **Week 3:** Basic LLM Chat → Simple patterns

**Goal:** Understand core concepts and build simple workflows

### 🚀 Intermediate (1-2 months)
1. **Month 1:** Streaming → Memory → Workflow Orchestration
2. **Month 2:** RAG System → Advanced error handling

**Goal:** Build production-ready AI applications

### 🌟 Advanced (Ongoing)
1. Agent Framework development
2. Meta-Flow experimentation
3. CosmoAI integration
4. Custom extensions

**Goal:** Contribute to CosmoAI ecosystem

---

## 🛠️ Development Guide

### Running Examples
```bash
# Basic examples (no external dependencies)
cargo run --example hello_world

# AI examples (requires API key)
export OPENAI_API_KEY=your_key
cargo run --example llm_chat --features "builtin-llm"

# Storage examples (requires services)
docker run -d -p 6379:6379 redis:latest
cargo run --example storage_showcase --features "storage-redis"
```

### Building Custom Examples
```rust
// Template for new examples
use pocketflow_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌌 PocketFlow-rs Example: Your Title Here");
    
    // Your example code here
    
    Ok(())
}
```

### Testing Examples
```bash
# Test all examples compile
cargo check --examples --all-features

# Run example tests
cargo test --examples --all-features
```

---

## 🔍 Troubleshooting

### Common Issues

**API Key Missing:**
```bash
export OPENAI_API_KEY=your_actual_api_key
# Or create .env file
echo "OPENAI_API_KEY=your_key" > .env
```

**Feature Not Enabled:**
```bash
# Enable required features
cargo run --example llm_chat --features "builtin-llm"
cargo run --example storage_redis --features "storage-redis"
```

**External Dependencies:**
```bash
# Start Redis for storage examples
docker run -d -p 6379:6379 redis:latest

# Start PostgreSQL for database examples
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:latest
```

### Getting Help

1. **Check example comments** - Each example has detailed inline documentation
2. **Review source code** - Examples are designed to be educational
3. **Run with debug logging** - `RUST_LOG=debug cargo run --example ...`
4. **Check feature flags** - Ensure required features are enabled

---

## 🤝 Contributing

Want to add an example? Follow these guidelines:

1. **Follow naming convention:** `##_descriptive_name.rs`
2. **Add comprehensive comments** explaining each concept
3. **Include error handling** and graceful failure
4. **Test with different configurations**
5. **Update this README** with your example

---

## 🚀 What's Next?

- **Explore the examples** in order of complexity
- **Experiment with modifications** to see how they affect behavior  
- **Combine patterns** to build your own applications
- **Share your creations** with the PocketFlow-rs community

**Welcome to the future of AI workflow development!** 🌌✨
