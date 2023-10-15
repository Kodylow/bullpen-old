use std::error::Error;

use bullpen::models::chat::chat_model::ChatModelTrait;
use bullpen::models::chat::ChatModels::ChatBison;
use bullpen::models::chat::{ChatExample, ChatMessage, ChatModel, ChatSession};
use bullpen::models::completion::completion_model::CompletionModelTrait;
use bullpen::models::completion::CompletionModels::TextBison;
use bullpen::models::embedding::embedding_model::EmbeddingModelTrait;
use bullpen::models::embedding::EmbeddingModels::TextEmbeddingGecko;
use bullpen::models::{CompletionModel, EmbeddingModel};
use futures_util::StreamExt;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::debug!("Debug logging is enabled");
    // -- Chat Model --
    let chat_model = ChatModel::new(ChatBison, None)?;
    let chat_session = ChatSession {
        context: "You are a programmer bot".to_string(),
        examples: vec![ChatExample {
            input: ChatMessage {
                content: "1 + 1".to_string(),
                author: "".to_string(),
            },
            output: ChatMessage {
                content: "2".to_string(),
                author: "".to_string(),
            },
        }],
        messages: vec![ChatMessage {
            content: "How do I write a nix flake for a rust project?".to_string(),
            author: "USER".to_string(),
        }],
    };

    let mut chat_stream = chat_model.stream_chat(vec![chat_session], 2000, 0.5).await;

    while let Some(chat_response) = chat_stream.next().await {
        info!("Model Response: {:?}", chat_response);
    }

    Ok(())
}
