use anyhow::Result;
use dotenvy::dotenv;
use openai::{
    Credentials,
    chat::{ChatCompletion, ChatCompletionDelta, ChatCompletionMessage, ChatCompletionMessageRole},
};
use serde_json::Value as JsonValue;
use std::io::{Write, stdout};
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

/// Configuration options for LLM streaming calls
#[derive(Default)]
pub struct StreamLlmOptions {
    /// Model name (if None, will be read from environment variables)
    pub model: Option<String>,
    /// Temperature parameter (0.0-2.0)
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u16>,
    /// Whether to print streaming output
    pub print_stream: bool,
}

/// Convert JSON messages to OpenAI ChatCompletionMessage format
pub fn convert_json_to_chat_messages(messages_json: &[JsonValue]) -> Vec<ChatCompletionMessage> {
    messages_json
        .iter()
        .map(|msg| {
            let role_str = msg["role"].as_str().unwrap_or("user");
            let role = match role_str {
                "assistant" => ChatCompletionMessageRole::Assistant,
                "system" => ChatCompletionMessageRole::System,
                _ => ChatCompletionMessageRole::User,
            };

            ChatCompletionMessage {
                role,
                content: Some(msg["content"].as_str().unwrap_or("").to_string()),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
            }
        })
        .collect()
}

/// Handle a completion stream and optionally display the tokens as they arrive
pub async fn handle_completion_stream(
    mut chat_stream: Receiver<ChatCompletionDelta>,
    print_output: bool,
) -> String {
    let mut merged: Option<ChatCompletionDelta> = None;

    // Print initial prompt if requested
    if print_output {
        print!("\nAssistant: ");
        stdout().flush().unwrap();
    }

    loop {
        match chat_stream.try_recv() {
            Ok(delta) => {
                let choice = &delta.choices[0];
                if let Some(content) = &choice.delta.content {
                    if print_output {
                        print!("{}", content);
                        stdout().flush().unwrap();
                    }
                }

                // Merge token into full completion
                match merged.as_mut() {
                    Some(c) => {
                        c.merge(delta).unwrap();
                    }
                    None => merged = Some(delta),
                };
            }
            Err(TryRecvError::Empty) => {
                let duration = std::time::Duration::from_millis(50);
                tokio::time::sleep(duration).await;
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
        };
    }

    if print_output {
        println!();
    }

    // Extract the full text response from the merged completion
    let chat_completion: ChatCompletion = match merged {
        Some(delta) => delta.into(),
        None => return String::new(), // Return an empty string if no completion is available
    };
    chat_completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .unwrap_or_default()
}

/// Call LLM with streaming and return the complete text response
pub async fn call_llm_streaming(
    messages: Vec<ChatCompletionMessage>,
    options: Option<StreamLlmOptions>,
) -> Result<String> {
    // Load API credentials
    dotenv().unwrap();
    let credentials = Credentials::from_env();

    let options = options.unwrap_or_default();
    let model = options
        .model
        .unwrap_or_else(|| std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()));

    // Build the streaming request with options
    let mut builder = ChatCompletionDelta::builder(&model, messages).credentials(credentials);

    if let Some(temp) = options.temperature {
        builder = builder.temperature(temp);
    }

    if let Some(max_tokens) = options.max_tokens {
        builder = builder.max_tokens(max_tokens);
    }

    // Create streaming request
    let chat_stream = builder.create_stream().await?;

    // Handle the stream and get the complete response
    let response = handle_completion_stream(chat_stream, options.print_stream).await;

    Ok(response)
}
