use std::error::Error;

use bullpen::models::chat::chat_model::ChatModelTrait;
use bullpen::models::chat::structs::Role;
use bullpen::models::chat::ChatModels::{self, ChatBison};
use bullpen::models::chat::{ChatExample, ChatMessage, ChatModel, ChatSession};
use bullpen::models::completion::completion_model::CompletionModelTrait;
use bullpen::models::completion::CompletionModels::TextBison;
use bullpen::models::CompletionModel;
use futures_util::StreamExt;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::debug!("Debug logging is enabled");
    // -- Chat Model --
    let chat_model = ChatModel::new(ChatModels::Llama2_13bChat, None)?;
    let chat_session = ChatSession {
        context: "You are a programmer bot".to_string(),
        examples: vec![ChatExample {
            input: ChatMessage {
                content: "1 + 1".to_string(),
                author: "user".to_string(),
            },
            output: ChatMessage {
                content: "2".to_string(),
                author: "assistant".to_string(),
            },
        }],
        messages: vec![ChatMessage {
            content: "How do I write a nix flake for a rust
    project?"
                .to_string(),
            author: "user".to_string(),
        }],
    };

    let chat_response = chat_model.chat(vec![chat_session], 200, 0.5).await?;

    info!("Model Response: {:?}", chat_response);

    // let mut chat_stream = chat_model.stream_chat(vec![chat_session], 200,
    // 0.5).await;

    // while let Some(chat_response) = chat_stream.next().await {
    //     info!("Model Response: {:?}", chat_response);
    // }

    // // -- Completion Model --
    // let completion_model = CompletionModel::new(TextBison, None)?;
    // let prompts = vec!["def add(a, b):".to_string()];
    // let mut completion_stream = completion_model.stream_complete(prompts,
    // 100, 0.5).await;

    // while let Some(completion_response) = completion_stream.next().await {
    //     info!("Model Response: {:?}", completion_response);
    // }

    Ok(())
}
