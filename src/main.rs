use std::error::Error;


use bullpen::models::chat::structs::Role;
use bullpen::models::chat::ChatModels::ChatBison;
use bullpen::models::chat::{ChatExample, ChatMessage, ChatModel, ChatSession};
use bullpen::models::completion::completion_model::CompletionModelTrait;
use bullpen::models::completion::CompletionModels::TextBison;


use bullpen::models::{CompletionModel};
use futures_util::StreamExt;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::debug!("Debug logging is enabled");
    // -- Chat Model --
    let _chat_model = ChatModel::new(ChatBison, None)?;
    let _chat_session = ChatSession {
        context: "You are a programmer bot".to_string(),
        examples: vec![ChatExample {
            input: ChatMessage {
                content: "1 + 1".to_string(),
                author: Role::User,
            },
            output: ChatMessage {
                content: "2".to_string(),
                author: Role::Assistant,
            },
        }],
        messages: vec![ChatMessage {
            content: "How do I write a nix flake for a rust
    project?"
                .to_string(),
            author: Role::User,
        }],
    };

    // let mut chat_stream = chat_model.stream_chat(vec![chat_session], 2000,
    // 0.5).await;

    // while let Some(chat_response) = chat_stream.next().await {
    //     info!("Model Response: {:?}", chat_response);
    // }

    // -- Completion Model --
    let completion_model = CompletionModel::new(TextBison, None)?;
    let prompts = vec!["def add(a, b):".to_string()];
    let mut completion_stream = completion_model.stream_complete(prompts, 100, 0.5).await;

    while let Some(completion_response) = completion_stream.next().await {
        info!("Model Response: {:?}", completion_response);
    }

    Ok(())
}
