use anyhow::Result;
use async_trait::async_trait;
use pocketflow_rs::core::{
    ExecResult, PostResult, PrepResult,
    communication::{BaseSharedStore, SharedStore},
    flow::Flow,
    node::NodeTrait,
};
use serde_json::{Value as JsonValue, json}; // Ensure JsonValue is in scope
use std::sync::Arc;

#[path = "../utils/mod.rs"]
mod utils;
use utils::call_llm_chat;

// Define an AnswerNode similar to the Python example
struct AnswerNode;

#[async_trait]
impl NodeTrait for AnswerNode {
    fn prep(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        // Extract the question from shared storage
        let question_json_val = shared_store
            .get_value("question") // Use trait method, returns Option<StoredValue>
            .ok_or_else(|| anyhow::anyhow!("Question StoredValue not found in shared store"))?;

        let question_json = question_json_val
            .downcast_ref::<JsonValue>() // Option<&JsonValue>
            .cloned() // Option<JsonValue>
            .ok_or_else(|| anyhow::anyhow!("Question StoredValue is not a JsonValue"))?;

        let question: String = serde_json::from_value(question_json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize question: {}", e))?;

        Ok(PrepResult::from(json!(question)))
    }

    fn exec(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        // Get the question from prep_res and call the LLM
        let question = prep_res
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Question not found in prep result"))?;

        let messages = vec![openai_api_rust::Message {
            role: openai_api_rust::Role::User,
            content: question.to_string(),
        }];
        let answer = call_llm_chat(&messages, None)?;

        Ok(ExecResult::from(json!(answer)))
    }

    fn post(
        &self,
        shared_store: &dyn SharedStore, // Use &dyn SharedStore trait
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        // Return PostResult
        let answer = exec_res
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Answer not found in exec result"))?;

        // Insert the answer into shared store using trait method
        shared_store.insert_value("answer", Arc::new(json!(answer)));
        Ok(PostResult::default()) // Return PostResult
    }
}

fn main() -> Result<()> {
    // Create a shared store with initial data
    let shared = BaseSharedStore::new_in_memory(); // Use BaseSharedStore for instantiation

    // Use BaseSharedStore's generic insert method for convenience here
    shared.insert(
        "question",
        json!("In one sentence, what's the end of universe?"),
    );

    // Create the node and flow
    let answer_node_arc = Arc::new(AnswerNode {}); // Arc<dyn NodeTrait>
    let qa_flow = Flow::new(Some(answer_node_arc));

    // Run the flow, passing &shared which implements &dyn SharedStore
    qa_flow.run(&shared)?;

    // Print the results using BaseSharedStore's generic get method
    println!(
        "Question: {}",
        shared // This is BaseSharedStore, so .get<T>() is available
            .get::<JsonValue>("question")
            .and_then(|jv| serde_json::from_value::<String>(jv).ok())
            .unwrap_or_else(|| "Question not found".to_string())
    );

    println!(
        "Answer: {}",
        shared // This is BaseSharedStore
            .get::<JsonValue>("answer")
            .and_then(|jv| serde_json::from_value::<String>(jv).ok())
            .unwrap_or_else(|| "Answer not found".to_string())
    );

    Ok(())
}
