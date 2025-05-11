use anyhow::Result;
use async_trait::async_trait;
use pocketflow_rs::{
    core::{
        ExecResult, PostResult, PrepResult,
        communication::{BaseSharedStore, SharedStore},
        flow::Flow,
        node::NodeTrait,
    },
    node::BaseNode,
};
use serde_json::{Value as JsonValue, json};
use std::{
    io::{self, Write},
    sync::Arc,
};
use utils::{StreamLlmOptions, call_llm_streaming, convert_json_to_chat_messages};

#[derive(Clone)]
struct StreamingChatNode {
    base: BaseNode,
}

#[async_trait]
impl NodeTrait for StreamingChatNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        self.base.prep(shared_store)
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        self.base.exec(prep_res)
    }

    fn post(
        &self,
        _shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        _exec_res: &ExecResult,
    ) -> Result<PostResult> {
        Ok(PostResult::default())
    }

    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        if !shared_store.contains_key("messages") {
            // Initialize the messages array with a system message
            let system_message = json!({
                "role": "system",
                "content": "You are a helpful assistant. Provide clear and concise responses."
            });

            let messages = vec![system_message];
            shared_store.insert_value("messages", Arc::new(json!(messages)));
            println!("Welcome to the streaming chat! Type 'exit' to end the conversation.");
        }

        print!("\nYou: ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim().to_string();

        // Get messages using trait method
        let messages_val = shared_store
            .get_value("messages")
            .expect("Messages StoredValue not found in prep")
            .downcast_ref::<JsonValue>()
            .cloned()
            .expect("Messages StoredValue is not JsonValue in prep");

        let mut messages_vec = messages_val
            .as_array()
            .expect("Messages JsonValue is not an array in prep")
            .clone();

        if user_input.to_lowercase() == "exit" {
            // If user types exit, prepare to exit
            return Ok(PrepResult::from(json!({"action": "exit"})));
        }

        messages_vec.push(json!({
            "role": "user",
            "content": user_input
        }));
        shared_store.insert_value("messages", Arc::new(JsonValue::Array(messages_vec.clone())));

        Ok(JsonValue::Array(messages_vec).into())
    }

    async fn exec_async(&self, prep_result: &PrepResult) -> Result<ExecResult> {
        // Check if prep_result signals to exit
        if let Some(obj) = prep_result.as_object() {
            if obj.get("action").and_then(|v| v.as_str()) == Some("exit") {
                return Ok(ExecResult::from(json!({"action": "exit"})));
            }
        }

        if prep_result.as_array().is_none() {
            return Ok(ExecResult::default());
        }

        let messages_json = prep_result.as_array().unwrap();

        // Convert messages to OpenAI format using utility function
        let messages = convert_json_to_chat_messages(messages_json);

        if messages.is_empty() {
            return Ok(ExecResult::default());
        }

        // Create streaming options
        let options = StreamLlmOptions {
            print_stream: true,
            ..Default::default()
        };

        // Call LLM with streaming using the utility function
        let response = call_llm_streaming(messages, Some(options)).await?;

        Ok(ExecResult::from(json!(response)))
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        _prep_result: &PrepResult,
        exec_result: &ExecResult,
    ) -> Result<PostResult> {
        // Check if exec_result signals to exit
        if let Some(obj) = exec_result.as_object() {
            if obj.get("action").and_then(|v| v.as_str()) == Some("exit") {
                println!("\nGoodbye!");
                return Ok(PostResult::default()); // Ends the loop
            }
        }

        let content = exec_result
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("LLM response not found in exec result"))?;

        // No need to print the content here since it was already printed in the stream handler

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

    // fn add_successor(&mut self, action: String, node: Arc<dyn NodeTrait>) {
    //     self.base.add_successor(action, node)
    // }

    // fn get_successor(&self, action: &str) -> Option<Arc<dyn NodeTrait>> {
    //     self.base.get_successor(action)
    // }
}

#[tokio::main]
async fn main() -> Result<()> {
    let chat_node = StreamingChatNode {
        base: BaseNode::new(),
    };

    let mut flow = Flow::new(Some(Arc::new(chat_node.clone())));
    flow.add_transition("continue".into(), Arc::new(chat_node));

    let shared = BaseSharedStore::new_in_memory();
    flow.run_async(&shared).await?; // Use run_async for async execution

    Ok(())
}
