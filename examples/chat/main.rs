use anyhow::Result;
use async_trait::async_trait;
use openai_api_rust::{Message, Role};
use pocketflow_rs::core::{
    ExecResult, PostResult, PrepResult,
    communication::{BaseSharedStore, SharedStore},
    flow::Flow,
    node::NodeTrait,
};
use serde_json::{Value as JsonValue, json};
use std::{
    io::{self, Write},
    sync::Arc,
};

#[path = "../utils/mod.rs"]
mod utils;
use utils::call_llm_chat;

struct ChatNode;

#[async_trait]
impl NodeTrait for ChatNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        if !shared_store.contains_key("messages") {
            // Use trait method for insert
            shared_store.insert_value("messages", Arc::new(json!([])));
            println!("Welcome to the chat! Type 'exit' to end the conversation.");
        }

        print!("\nYou: ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim().to_string();

        if user_input.to_lowercase() == "exit" {
            // To signal flow to stop, PostResult should be empty or a specific "stop" value
            // that get_successor won't match, or prep itself could return an error/specific result.
            // For now, returning default PrepResult and relying on post to return non-looping action.
            // A better way would be for prep to indicate exit, e.g. by returning Err or specific PrepResult.
            // Let's assume "exit" input means the 'post' phase will return a non-"continue" action.
            // Or, we can make prep return a special value that exec/post can check.
            // For now, we'll pass "exit" through, and post will handle it.
        }

        // Get messages using trait method
        let messages_val = shared_store
            .get_value("messages")
            .expect("Messages StoredValue not found in prep") // Should exist due to init above
            .downcast_ref::<JsonValue>()
            .cloned()
            .expect("Messages StoredValue is not JsonValue in prep");

        let mut messages_vec = messages_val
            .as_array()
            .expect("Messages JsonValue is not an array in prep")
            .clone();

        if user_input.to_lowercase() == "exit" {
            // If user types exit, we don't add it to history, just prepare to exit.
            // We need a way for exec/post to know this. Let's pass a special PrepResult.
            return Ok(PrepResult::from(json!({"action": "exit"})));
        }

        messages_vec.push(json!({
            "role": "user",
            "content": user_input
        }));
        shared_store.insert_value("messages", Arc::new(JsonValue::Array(messages_vec.clone())));

        Ok(JsonValue::Array(messages_vec).into())
    }

    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        // Check if prep_result signals to exit
        if let Some(obj) = prep_result.as_object() {
            if obj.get("action").and_then(|v| v.as_str()) == Some("exit") {
                return Ok(ExecResult::from(json!({"action": "exit"}))); // Pass exit signal
            }
        }

        if prep_result.as_array().is_none() {
            // This case might be hit if prep_result was {"action": "exit"} and not an array
            return Ok(ExecResult::default());
        }

        let messages_json = prep_result.as_array().unwrap(); // Already checked above, or should be an array if not exit

        let messages_for_llm = messages_json
            .iter()
            .map(|msg| {
                let role_str = msg["role"].as_str().unwrap_or("user");
                let role = match role_str {
                    "assistant" => Role::Assistant,
                    "system" => Role::System,
                    _ => Role::User,
                };
                let content = msg["content"].as_str().unwrap_or("").to_string();
                Message { role, content }
            })
            .collect::<Vec<_>>();

        if messages_for_llm.is_empty() {
            // Should not happen if not exiting
            return Ok(ExecResult::default());
        }

        let response = call_llm_chat(&messages_for_llm, None)?;
        Ok(ExecResult::from(json!(response)))
    }

    fn post(
        &self,
        shared_store: &dyn SharedStore,
        _prep_result: &PrepResult, // prep_result might be used to check for "exit" signal
        exec_result: &ExecResult,
    ) -> Result<PostResult> {
        // Check if exec_result signals to exit
        if let Some(obj) = exec_result.as_object() {
            if obj.get("action").and_then(|v| v.as_str()) == Some("exit") {
                println!("\nGoodbye!");
                return Ok(PostResult::default()); // Ends the loop as get_successor for "" is None
            }
        }

        let content = exec_result
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("LLM response not found in exec result"))?;

        println!("\nAssistant: {}", content);

        let messages_val = shared_store
            .get_value("messages")
            .expect("Messages StoredValue not found in post")
            .downcast_ref::<JsonValue>()
            .cloned()
            .expect("Messages StoredValue is not JsonValue in post");

        let mut messages_vec = messages_val
            .as_array()
            .expect("Messages JsonValue is not an array in post")
            .clone();

        messages_vec.push(json!({
            "role": "assistant",
            "content": content
        }));
        shared_store.insert_value("messages", Arc::new(JsonValue::Array(messages_vec)));

        Ok(PostResult::from("continue"))
    }

    // For self-looping, ChatNode needs to return itself when action is "continue"
    // This requires ChatNode to have access to an Arc of itself.
    // This is a more complex pattern (e.g. Arc::new_cyclic or passing Arc<Self> in constructor).
    // For now, to make it compile and run once:
    fn get_successor(&self, action: &str) -> Option<Arc<dyn NodeTrait>> {
        if action == "continue" {
            // This would create a new instance, not loop on the same one.
            // To truly loop, the Arc<ChatNode> from main needs to be accessible here.
            // For this example, let's make it loop by creating a new instance.
            // This is NOT a persistent chat session across loops with this simple implementation.
            // A proper implementation would involve a way to pass Arc<Self> or use Weak pointers.
            Some(Arc::new(ChatNode {}))
        } else {
            None
        }
    }
}

fn main() -> Result<()> {
    let chat_node_arc = Arc::new(ChatNode {}); // Arc<dyn NodeTrait>

    // Flow::new expects Option<Arc<dyn NodeTrait>>
    // The old flow.add_transition("continue", chat_node_arc.clone()) is replaced by
    // ChatNode's get_successor method.
    let flow = Flow::new(Some(chat_node_arc));

    let shared = BaseSharedStore::new_in_memory();
    flow.run(&shared)?; // Pass &shared which impls &dyn SharedStore

    Ok(())
}
