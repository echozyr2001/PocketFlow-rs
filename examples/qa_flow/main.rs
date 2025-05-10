use anyhow::Result;
use async_trait::async_trait;
use openai::chat::{ChatCompletionMessage, ChatCompletionMessageRole};
use pocketflow_rs::core::{
    ExecResult, PostResult, PrepResult,
    communication::{BaseSharedStore, SharedStore},
    flow::Flow,
    node::NodeTrait,
};
use serde_json::{Value as JsonValue, json};
use std::sync::Arc;
use utils::{StreamLlmOptions, call_llm_streaming};

struct QANode;

#[async_trait]
impl NodeTrait for QANode {
    async fn prep_async(&self, shared_store: &dyn SharedStore) -> Result<PrepResult> {
        // Extract the question from shared storage
        let question_json_val = shared_store
            .get_value("question")
            .ok_or_else(|| anyhow::anyhow!("Question StoredValue not found in shared store"))?;

        let question_json = question_json_val
            .downcast_ref::<JsonValue>()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Question StoredValue is not a JsonValue"))?;

        let question: String = serde_json::from_value(question_json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize question: {}", e))?;

        Ok(PrepResult::from(json!(question)))
    }

    async fn exec_async(&self, prep_res: &PrepResult) -> Result<ExecResult> {
        // Get the question from prep_res and stream the LLM response
        let question = prep_res
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Question not found in prep result"))?;

        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some("You provide concise and informative answers.".to_string()),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(question.to_string()),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        // Create streaming options with printing enabled
        let options = StreamLlmOptions {
            print_stream: true,
            ..Default::default()
        };

        // Call LLM with streaming using the utility function
        let answer = call_llm_streaming(messages, Some(options)).await?;

        Ok(ExecResult::from(json!(answer)))
    }

    async fn post_async(
        &self,
        shared_store: &dyn SharedStore,
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<PostResult> {
        // Return PostResult
        let answer = exec_res
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Answer not found in exec result"))?;

        // Insert the answer into shared store using trait method
        shared_store.insert_value("answer", Arc::new(json!(answer)));
        Ok(PostResult::default())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a shared store with initial data
    let shared = BaseSharedStore::new_in_memory();

    const QUESTION: &str = "In one sentence, what is the end of universe?";
    println!("Question: {}", QUESTION);

    // Use BaseSharedStore's generic insert method for convenience here
    shared.insert("question", json!(QUESTION));

    // Create the node and flow
    let qa_node_arc = Arc::new(QANode {});
    let qa_flow = Flow::new(Some(qa_node_arc));

    // Run the flow asynchronously
    qa_flow.run_async(&shared).await?;

    Ok(())
}
