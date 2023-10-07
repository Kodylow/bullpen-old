use std::error::Error;

use bullpen::models::chat::chat_model::ChatModelTrait;
use bullpen::models::chat::ChatModels::ChatBison;
use bullpen::models::chat::{ChatExample, ChatMessage, ChatModel, ChatSession};
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

    // // -- Embedding Model --
    // let embedding_model = EmbeddingModel::new(TextEmbeddingGecko, None)?;

    // let embeddings_content = vec![
    //     "Hello world!".to_string(),
    //     "Replit Modelfarm in Rust!".to_string(),
    // ];

    // let embeddings = embedding_model.embed(embeddings_content).await?;

    // println!("Embeddings: {:?}", embeddings);

    // // -- Completion Model --
    // let completion_model = CompletionModel::new(TextBison, None)?;

    // let prompts = vec!["Hello world!".to_string()];

    // let mut completion_stream = completion_model.stream_complete(prompts, 10,
    // 0.5).await?;

    // while let Some(completion_response) = completion_stream.next().await {
    //     println!("Model Response: {:?}", completion_response);
    // }

    Ok(())
}
