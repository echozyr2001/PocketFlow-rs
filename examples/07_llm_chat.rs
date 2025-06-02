//! 🤖 PocketFlow-rs LLM Chat Integration
//!
//! Basic AI chat integration simulation.
//! Demonstrates how LLM concepts can be integrated into PocketFlow-rs workflows.

#[cfg(feature = "builtin-llm")]
use pocketflow_rs::prelude::*;
#[cfg(feature = "builtin-llm")]
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 PocketFlow-rs LLM Chat Integration");
    println!("Integrating AI chat capabilities into workflows\n");

    // Check if LLM features are available
    #[cfg(feature = "builtin-llm")]
    {
        llm_integration_example().await?;
    }

    #[cfg(not(feature = "builtin-llm"))]
    {
        println!("⚠️  LLM features not enabled!");
        println!("Run with: cargo run --example 07_llm_chat --features builtin-llm");

        // Show a conceptual example without LLM features
        conceptual_example().await?;
    }

    Ok(())
}

#[cfg(feature = "builtin-llm")]
async fn llm_integration_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 LLM Integration Patterns with PocketFlow-rs\n");

    // Pattern 1: Simple Chat Flow
    println!("📝 Pattern 1: Simple Chat Processing");

    let mut simple_chat = FlowBuilder::new()
        .start_node("input")
        .terminal_action("complete")
        .node(
            "input",
            Node::new(SetValueNode::new(
                "user_input".to_string(),
                json!("Hello, tell me about PocketFlow-rs"),
                Action::simple("process"),
            )),
        )
        .node(
            "chat_sim",
            Node::new(LogNode::new(
                "🤖 Processing chat request...",
                Action::simple("respond"),
            )),
        )
        .node(
            "response",
            Node::new(SetValueNode::new(
                "ai_response".to_string(),
                json!(
                    "PocketFlow-rs is a powerful workflow orchestration engine for AI applications!"
                ),
                Action::simple("complete"),
            )),
        )
        .route("input", "process", "chat_sim")
        .route("chat_sim", "respond", "response")
        .build();

    let mut store = SharedStore::new();
    let result = simple_chat.execute(&mut store).await?;

    println!("💬 Chat Simulation Result:");
    if let Some(input) = store.get("user_input")? {
        println!("👤 User: {}", input.as_str().unwrap_or(""));
    }
    if let Some(response) = store.get("ai_response")? {
        println!("🤖 AI: {}", response.as_str().unwrap_or(""));
    }
    println!("📊 Steps executed: {}", result.steps_executed);

    println!("{}", "\n".to_owned() + &"=".repeat(60));

    // Pattern 2: Multi-turn Conversation
    println!("\n🔄 Pattern 2: Multi-turn Conversation Flow");

    for (turn, question) in [
        "What is PocketFlow-rs?",
        "How does it work?",
        "Give me an example",
    ]
    .iter()
    .enumerate()
    {
        println!("\n💭 Turn {}: {}", turn + 1, question);

        let mut turn_flow = FlowBuilder::new()
            .start_node("question")
            .terminal_action("answered")
            .node("question", Node::new(SetValueNode::new(
                "current_question".to_string(),
                json!(question),
                Action::simple("thinking")
            )))
            .node("thinking", Node::new(DelayNode::new(
                std::time::Duration::from_millis(100),
                Action::simple("respond")
            )))
            .node("respond", Node::new(SetValueNode::new(
                "response".to_string(),
                json!(match turn {
                    0 => "PocketFlow-rs is a workflow orchestration engine for AI applications",
                    1 => "It uses a three-phase execution model: prep, exec, post",
                    _ => "You can build complex AI workflows with conditional logic and data flow"
                }),
                Action::simple("answered")
            )))
            .route("question", "thinking", "thinking")
            .route("thinking", "respond", "respond")
            .build();

        let mut turn_store = SharedStore::new();
        turn_store.set("turn".to_string(), json!(turn + 1))?;

        let _turn_result = turn_flow.execute(&mut turn_store).await?;

        if let Some(response) = turn_store.get("response")? {
            println!("🤖 AI: {}", response.as_str().unwrap_or(""));
        }
    }

    println!("{}", "\n".to_owned() + &"=".repeat(60));

    // Pattern 3: Context-Aware Processing
    println!("\n🧠 Pattern 3: Context-Aware Processing");

    let mut context_flow = FlowBuilder::new()
        .start_node("init")
        .terminal_action("complete")
        .max_steps(10)
        .node("init", Node::new(SetValueNode::new(
            "context".to_string(),
            json!({"conversation_history": [], "user_preferences": "technical"}),
            Action::simple("ready")
        )))
        .node("analyze", Node::new(ConditionalNode::new(
            |store| {
                if let Ok(Some(context)) = store.get("context") {
                    if let Some(prefs) = context.get("user_preferences") {
                        return prefs.as_str().unwrap_or("") == "technical";
                    }
                }
                false
            },
            Action::simple("technical"),
            Action::simple("simple")
        )))
        .node("technical_response", Node::new(SetValueNode::new(
            "final_response".to_string(),
            json!("PocketFlow-rs implements a directed acyclic graph (DAG) execution model with shared state management"),
            Action::simple("complete")
        )))
        .node("simple_response", Node::new(SetValueNode::new(
            "final_response".to_string(),
            json!("PocketFlow-rs helps you build smart workflows easily!"),
            Action::simple("complete")
        )))
        .route("init", "ready", "analyze")
        .route("analyze", "technical", "technical_response")
        .route("analyze", "simple", "simple_response")
        .build();

    let mut context_store = SharedStore::new();
    let context_result = context_flow.execute(&mut context_store).await?;

    println!("🎯 Context-aware response:");
    if let Some(response) = context_store.get("final_response")? {
        println!("🤖 AI: {}", response.as_str().unwrap_or(""));
    }
    println!("📊 Execution path: {:?}", context_result.execution_path);

    println!("\n💡 LLM Integration Best Practices:");
    println!("  🔧 Use SetValueNode for prompt/response management");
    println!("  🎭 Use ConditionalNode for dynamic response selection");
    println!("  📚 Store conversation context in SharedStore");
    println!("  ⏱️  Use DelayNode to simulate processing time");
    println!("  🔄 Design for multi-turn conversations");
    println!("  🎯 Implement context-aware decision making");

    Ok(())
}

#[cfg(not(feature = "builtin-llm"))]
async fn conceptual_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("💡 Conceptual LLM Integration with PocketFlow-rs\n");
    println!("🎯 This example shows the patterns you would use for LLM integration:");

    println!(
        r#"
🔄 Multi-step LLM Workflow Pattern:

   Input → Context → LLM Call → Process → Output
     ↓       ↓         ↓         ↓        ↓
   Store   Prepare   Execute   Parse    Store
   Query   Prompt    Request   Result   Response

🧩 Key Components:
  📝 SetValueNode: Store prompts and responses
  🤔 ConditionalNode: Choose response strategies  
  ⏱️  DelayNode: Rate limiting and processing time
  🔄 Flow routing: Handle multi-turn conversations

🎯 Implementation Strategy:
  1. Use SharedStore for conversation state
  2. Build modular nodes for each LLM operation
  3. Design for error handling and retries
  4. Implement context management
  5. Support streaming and async operations

💡 Enable with: --features builtin-llm
"#
    );

    Ok(())
}
