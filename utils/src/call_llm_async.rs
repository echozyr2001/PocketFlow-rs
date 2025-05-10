use anyhow::Result;
use dotenvy::dotenv;
use openai::{
    Credentials,
    chat::{ChatCompletion, ChatCompletionMessage},
};

pub async fn call_llm_async(messages: &[ChatCompletionMessage]) -> Result<String> {
    dotenv().unwrap();
    let credentials = Credentials::from_env();
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

    let chat_completion = ChatCompletion::builder(&model, messages)
        .credentials(credentials.clone())
        .create()
        .await
        .unwrap();

    let returned_message = chat_completion.choices.first().unwrap().message.clone();

    Ok(returned_message.content.unwrap().trim().to_string())
}
