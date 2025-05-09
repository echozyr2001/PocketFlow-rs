use anyhow::Result;
use openai_api_rust::{Message, Role};
use pocketflow_rs::core::{
    Action, ExecResult, PrepResult, communication::SharedStore, flow::Flow, node::BaseNode,
};
use serde_json::json;
use std::{
    io::{self, Write},
    sync::Arc,
};

#[path = "../utils/mod.rs"]
mod utils;
use utils::call_llm_chat;

struct ChatNode;

impl BaseNode for ChatNode {
    fn prep(&self, shared: &SharedStore) -> Result<PrepResult> {
        // Initialize messages if this is the first run
        if !shared.contains_key("messages") {
            shared.insert("messages", json!([]));
            println!("Welcome to the chat! Type 'exit' to end the conversation.");
        }

        // Get user input
        print!("\nYou: ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim().to_string();

        // Check if user wants to exit
        if user_input.to_lowercase() == "exit" {
            return Ok(PrepResult::default());
        }

        // Add user message to history
        let mut messages = shared
            .get::<serde_json::Value>("messages")
            .expect("Messages JsonValue not found or wrong type in prep")
            .as_array()
            .expect("Messages JsonValue is not an array in prep")
            .clone();
        messages.push(json!({
            "role": "user",
            "content": user_input
        }));
        shared.insert("messages", json!(messages));

        // Return all messages for the LLM
        Ok(json!(messages).into())
    }

    fn exec(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        // Check if the prep result contains data
        if prep_result.as_object().is_none() && prep_result.as_array().is_none() {
            return Ok(ExecResult::default());
        }

        let messages = prep_result
            .as_array()
            .unwrap()
            .iter()
            .map(|msg| {
                let role = match msg["role"].as_str().unwrap() {
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    "system" => Role::System,
                    _ => Role::User,
                };

                let content = msg["content"].as_str().unwrap().to_string();
                Message { role, content }
            })
            .collect::<Vec<_>>();
        let response = call_llm_chat(&messages, None)?;

        Ok(ExecResult::from(json!(response)))
    }

    fn post(
        &self,
        shared: &SharedStore,
        prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<Action> {
        // Check if we have valid results
        if prep_result.as_array().is_none() || exec_result.as_str().is_none() {
            println!("\nGoodbye!");
            return Ok(Action::default()); // End the conversation
        }

        // Extract assistant's response
        let content = exec_result.as_str().unwrap();

        // Print the assistant's response
        println!("\nAssistant: {}", content);

        // Add assistant message to history
        let mut messages = shared
            .get::<serde_json::Value>("messages")
            .expect("Messages JsonValue not found or wrong type in post")
            .as_array()
            .expect("Messages JsonValue is not an array in post")
            .clone();
        messages.push(json!({
            "role": "assistant",
            "content": content
        }));
        shared.insert("messages", json!(messages)); // No ?

        // Loop back to continue the conversation
        Ok("continue".into())
    }
}

fn main() -> Result<()> {
    // Create the flow with self-loop
    let chat_node = Arc::new(ChatNode);

    let mut flow = Flow::new(Some(chat_node.clone()));
    flow.add_transition("continue", chat_node);

    // Start the chat
    let shared = SharedStore::new();
    flow.run(&shared)?;

    Ok(())
}
