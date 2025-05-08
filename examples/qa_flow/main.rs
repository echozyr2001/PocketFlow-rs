use anyhow::Result;
use pocketflow_rs::core::{
    Action, ExecResult, PrepResult, communication::SharedStore, flow::Flow, node::BaseNode,
};
use serde_json::json;
use std::sync::Arc;

#[path = "../utils/mod.rs"]
mod utils;
use utils::call_llm_chat;

// Define an AnswerNode similar to the Python example
struct AnswerNode;

impl BaseNode for AnswerNode {
    fn prep(&self, shared: &SharedStore) -> Result<PrepResult> {
        // Extract the question from shared storage
        let question = shared
            .get_json::<String>("question")
            .ok_or_else(|| anyhow::anyhow!("Question not found in shared store"))?;

        // Convert the question string to JSON value and create PrepResult
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

        // Convert the answer string to JSON value and create ExecResult
        Ok(ExecResult::from(json!(answer)))
    }

    fn post(
        &self,
        shared: &SharedStore,
        _prep_res: &PrepResult,
        exec_res: &ExecResult,
    ) -> Result<Action> {
        // Store the answer in shared
        let answer = exec_res
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Answer not found in exec result"))?;

        // Insert the answer into shared store
        shared.insert_json("answer", answer);
        Ok(Action::default())
    }
}

fn main() -> Result<()> {
    // Create a shared store with initial data
    let shared = SharedStore::new();
    shared.insert_json("question", "In one sentence, what's the end of universe?");

    // Create the node and flow
    let answer_node = Arc::new(AnswerNode);
    let qa_flow = Flow::new(Some(answer_node));

    // Run the flow
    qa_flow.run(&shared)?;

    // Print the results
    println!(
        "Question: {}",
        shared
            .get_json::<String>("question")
            .unwrap_or_else(|| "Question not found".to_string())
    );

    println!(
        "Answer: {}",
        shared
            .get_json::<String>("answer")
            .unwrap_or_else(|| "Answer not found".to_string())
    );

    Ok(())
}
