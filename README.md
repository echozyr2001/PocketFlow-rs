# PocketFlow-rs

Reference from [PocketFlow](https://github.com/The-Pocket/PocketFlow)

## Table of Contents

- [Core Concepts](#core-concepts)
- [API Reference](#api-reference)
- [Examples](#examples)

## Core Concepts

### Node System

Nodes are the fundamental building blocks of PocketFlow workflows. Each node follows a three-phase execution model:

#### 1. Built-in Nodes

PocketFlow-rs provides several built-in nodes for common operations:

```rust
use pocketflow_rs::prelude::*;
use std::time::Duration;

// Logging and debugging
let log_node = LogNode::new("Processing started", Action::simple("continue"));

// Data manipulation
let set_node = SetValueNode::new(
    "result".to_string(),
    json!("success"),
    Action::simple("stored")
);

let get_node = GetValueNode::new(
    "input".to_string(),
    "output".to_string(),
    |value| value.unwrap_or(json!("default")),
    Action::simple("retrieved")
);

// Conditional logic
let conditional_node = ConditionalNode::new(
    |store: &SharedStore<_>| store.contains_key("ready").unwrap_or(false),
    Action::simple("proceed"),
    Action::simple("wait")
);

// Delays and timing
let delay_node = DelayNode::new(
    Duration::from_secs(1),
    Action::simple("delay_complete")
);

// Mock LLM for testing
let llm_node = MockLlmNode::new(
    "prompt".to_string(),
    "response".to_string(),
    "Mock AI response".to_string(),
    Action::simple("llm_complete")
).with_retries(3);

// API requests
let api_node = ApiRequestNode::new(
    ApiConfig::default(),
    "prompt".to_string(),
    "response".to_string(),
    Action::simple("api_complete")
);
```

#### 2. Custom Nodes

Create custom nodes by implementing the `NodeBackend` trait:

```rust
use pocketflow_rs::prelude::*;
use async_trait::async_trait;

struct CustomProcessingNode {
    multiplier: f64,
}

#[async_trait]
impl NodeBackend<InMemoryStorage> for CustomProcessingNode {
    type PrepResult = f64;
    type ExecResult = f64;
    type Error = NodeError;
    
    async fn prep(&mut self, store: &SharedStore<InMemoryStorage>, _context: &ExecutionContext) 
        -> Result<Self::PrepResult, Self::Error> {
        let value = store.get("number")?
            .and_then(|v| v.as_f64())
            .ok_or(NodeError::ValidationError("Number not found".to_string()))?;
        Ok(value)
    }
    
    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) 
        -> Result<Self::ExecResult, Self::Error> {
        Ok(prep_result * self.multiplier)
    }
    
    async fn post(&mut self, store: &mut SharedStore<InMemoryStorage>, 
                  _prep_result: Self::PrepResult, exec_result: Self::ExecResult, 
                  _context: &ExecutionContext) -> Result<Action, Self::Error> {
        store.set("result".to_string(), json!(exec_result))?;
        Ok(Action::simple("complete"))
    }
}
```

#### 3. Function Nodes

For quick prototyping, use `FunctionNode`:

```rust
let function_node = FunctionNode::new(
    "QuickProcessor".to_string(),
    // Prep
    |store: &SharedStore<_>, _| store.get("input").ok().flatten().unwrap_or(json!(0)),
    // Exec
    |input, _| Ok(json!(input.as_i64().unwrap_or(0) * 2)),
    // Post
    |store: &mut SharedStore<_>, _, result, _| {
        store.set("output".to_string(), result)?;
        Ok(Action::simple("done"))
    }
);
```

### Action System

PocketFlow-rs features a rich action system supporting various action types:

```rust
use pocketflow_rs::prelude::*;

// Simple actions
let action = Action::simple("continue");

// Parameterized actions
let action = Action::parameterized("process", vec![
    ("model".to_string(), "gpt-4".to_string()),
    ("temperature".to_string(), "0.7".to_string()),
]);

// Conditional actions
let action = Action::conditional(
    ActionCondition::and(vec![
        ActionCondition::equals("status", "ready"),
        ActionCondition::greater_than("confidence", "0.8"),
    ]),
    Action::simple("proceed"),
    Action::simple("retry")
);

// Multiple actions
let action = Action::multiple(vec![
    Action::simple("validate"),
    Action::simple("process"),
    Action::simple("save"),
]);

// Actions with metadata
let action = Action::with_metadata(
    Action::simple("llm_call"),
    vec![
        ("model".to_string(), "gpt-4".to_string()),
        ("timestamp".to_string(), "2024-01-01T00:00:00Z".to_string()),
    ]
);
```

### Flow System

Flows orchestrate node execution through actions and routing:

#### Basic Flow

```rust
let mut flow = FlowBuilder::new()
    .start_node("start")
    .node("start", start_node)
    .node("process", process_node)
    .node("end", end_node)
    .route("start", "init", "process")
    .route("process", "complete", "end")
    .build();
```

#### Conditional Routing

```rust
let mut flow = FlowBuilder::new()
    .start_node("check")
    .conditional_route(
        "check", 
        "evaluate", 
        "success", 
        RouteCondition::KeyEquals("status".to_string(), json!("ok"))
    )
    .conditional_route(
        "check", 
        "evaluate", 
        "failure", 
        RouteCondition::KeyEquals("status".to_string(), json!("error"))
    )
    .build();
```

#### Nested Flows

```rust
// Create sub-flows
let validation_flow = FlowBuilder::new()
    .start_node("validate")
    .node("validate", validation_node)
    .build();

// Use as a node in main flow
let mut main_flow = FlowBuilder::new()
    .start_node("input")
    .node("input", input_node)
    .node("validation", Node::new(FlowNode::new(validation_flow)))
    .route("input", "to_validation", "validation")
    .build();
```

### Storage System

PocketFlow-rs supports multiple storage backends:

#### In-Memory Storage (Default)

```rust
let mut store = SharedStore::new(); // Uses InMemoryStorage
store.set("key".to_string(), json!("value"))?;
```

#### File-Based Storage

```rust
let file_storage = FileStorage::new("./data.json")?;
let mut store = SharedStore::with_storage(file_storage);
store.set("persistent_key".to_string(), json!("persisted_value"))?;
```

#### Custom Storage

Implement the `StorageBackend` trait for custom storage solutions:

```rust
use async_trait::async_trait;

struct DatabaseStorage {
    // Database connection
}

#[async_trait]
impl StorageBackend for DatabaseStorage {
    async fn get(&self, key: &str) -> Result<Option<JsonValue>, Box<dyn std::error::Error + Send + Sync>> {
        // Implement database get
        todo!()
    }
    
    async fn set(&mut self, key: String, value: JsonValue) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implement database set
        todo!()
    }
    
    // ... other methods
}
```

## API Reference

### Core Types

- **`SharedStore<S: StorageBackend>`**: Thread-safe key-value store
- **`Node<B: NodeBackend<S>, S: StorageBackend>`**: Workflow execution unit
- **`Flow`**: Workflow orchestrator
- **`Action`**: Rich action type with conditions and metadata
- **`StorageBackend`**: Trait for storage implementations

### Built-in Nodes

- **`LogNode`**: Logging and debugging
- **`SetValueNode`**: Store values in SharedStore
- **`GetValueNode`**: Retrieve and transform values
- **`ConditionalNode`**: Conditional branching
- **`DelayNode`**: Add delays to workflows
- **`MockLlmNode`**: Mock LLM for testing
- **`ApiRequestNode`**: Make API requests to LLM services

### Storage Backends

- **`InMemoryStorage`**: Fast in-memory storage
- **`RedisStorage`**: Redis-backed persistent storage (enable with `redis` feature)
- **`DatabaseStorage`**: SQL database storage using SeaORM (enable with `database` feature)
  - Supports SQLite, PostgreSQL, MySQL with granular feature control:
    - `database-sqlite`: SQLite support only
    - `database-postgres`: PostgreSQL support only  
    - `database-mysql`: MySQL support only
    - `database`: All database backends

## Examples

### 1. Basic Usage

```bash
cargo run --example basic_usage
```

Demonstrates basic SharedStore operations and storage backends.

### 2. Enhanced Actions

```bash
cargo run --example enhanced_actions
```

Shows the rich action system with conditions and parameters.

### 3. Node System

```bash
cargo run --example node_system
```

Explores built-in nodes and custom node creation.

### 4. Flow System

```bash
cargo run --example flow_system
```

Demonstrates flow creation, routing, and execution.

### 5. Nested Flows

```bash
cargo run --example nested_flows
```

Shows how to compose complex workflows using nested flows.

### 6. API Requests

```bash
cargo run --example api_request
```

Demonstrates integration with OpenAI API and error handling.

### 7. Redis Storage

```bash
# First, start Redis (requires Docker or local Redis installation)
docker run --rm -p 6379:6379 redis:latest

# Then run the Redis example 
cargo run --example redis_storage --features redis
```

Shows how to use Redis as a persistent storage backend for distributed workflows.

### 8. Database Storage

```bash
# SQLite (lightweight, serverless)
cargo run --example database_storage --features database-sqlite

# PostgreSQL (advanced features, JSONB support)
cargo run --example postgres_storage --features database-postgres  

# MySQL (web applications, e-commerce)
cargo run --example mysql_storage --features database-mysql

# All databases
cargo run --example database_storage --features database
```

Demonstrates SeaORM database integration with specific database examples showcasing each database's strengths and use cases.

### Real-World Patterns

#### RAG (Retrieval-Augmented Generation)

```rust
// Document retrieval node
let retrieve_node = Node::new(FunctionNode::new(
    "DocumentRetriever".to_string(),
    |store: &SharedStore<_>, _| {
        store.get("query").ok().flatten().unwrap_or(json!(""))
    },
    |query, _| {
        // Simulate document retrieval
        Ok(json!(["doc1", "doc2", "doc3"]))
    },
    |store: &mut SharedStore<_>, _, docs, _| {
        store.set("documents".to_string(), docs)?;
        Ok(Action::simple("to_generate"))
    }
));

// LLM generation node
let generate_node = Node::new(ApiRequestNode::new(
    api_config,
    "query".to_string(),
    "response".to_string(),
    Action::simple("complete")
).with_system_message("Use the provided documents to answer the question."));

// RAG flow
let mut rag_flow = FlowBuilder::new()
    .start_node("retrieve")
    .node("retrieve", retrieve_node)
    .node("generate", generate_node)
    .route("retrieve", "to_generate", "generate")
    .build();
```

#### Multi-Agent System

```rust
// Agent nodes with different roles
let researcher = Node::new(ApiRequestNode::new(
    config.clone(),
    "task".to_string(),
    "research_result".to_string(),
    Action::simple("to_analyst")
).with_system_message("You are a research agent. Gather information."));

let analyst = Node::new(ApiRequestNode::new(
    config.clone(),
    "research_result".to_string(),
    "analysis".to_string(),
    Action::simple("to_writer")
).with_system_message("You are an analyst. Analyze the research."));

let writer = Node::new(ApiRequestNode::new(
    config,
    "analysis".to_string(),
    "final_report".to_string(),
    Action::simple("complete")
).with_system_message("You are a writer. Create a final report."));

// Multi-agent flow
let mut agent_flow = FlowBuilder::new()
    .start_node("researcher")
    .node("researcher", researcher)
    .node("analyst", analyst)
    .node("writer", writer)
    .route("researcher", "to_analyst", "analyst")
    .route("analyst", "to_writer", "writer")
    .build();
```