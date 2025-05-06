use anyhow::Result;
use dotenvy::dotenv;
use openai_api_rust::{
    Auth, Message, OpenAI, Role,
    chat::{ChatApi, ChatBody},
};
use serde_json::json;
use std::sync::Arc;

// Import required components from the pocketflow-rs crate
use pocketflow_rs::core::{
    Action, ExecResult, PrepResult, communication::SharedStore, flow::Flow, node::BaseNode,
};

fn call_llm_real(prompt: &str) -> Result<String> {
    // Make sure you have a file named `.env` with the `OPENAI_KEY` environment variable defined!
    dotenv().unwrap();
    let api_key =
        std::env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("Missing OPENAI_API_KEY"))?;

    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1/".to_string());

    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

    let auth = Auth::new(&api_key);

    let openai = OpenAI::new(auth, &base_url);
    let body = ChatBody {
        model,
        max_tokens: None,
        temperature: None,
        top_p: None,
        n: None,
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message {
            role: Role::User,
            content: prompt.into(),
        }],
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();

    Ok(message.content.clone())
}

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

        let answer = call_llm_real(question)?;

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
