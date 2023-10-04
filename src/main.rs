use bullpen::{
    completion_model::CompletionModel,
    embedding_model::EmbeddingModel,
    structs::{ChatExample, ChatMessage, ChatSession},
    ChatModel,
};
use futures_util::StreamExt;
use serde_json::Value;
use std::{collections::HashMap, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // -- Chat Model --
    let chat_model = ChatModel::new("chat-bison", None)?;
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
            content: "What is Replit?".to_string(),
            author: "USER".to_string(),
        }],
    };

    let mut chat_stream = chat_model.stream_chat(vec![chat_session], 10, 0.5).await?;

    while let Some(chat_response) = chat_stream.next().await {
        println!("Model Response: {:?}", chat_response);
    }
    

    // -- Embedding Model --
    let embedding_model = EmbeddingModel::new("textembedding-gecko", None)?;

    let mut content = HashMap::<String, Value>::new();
    content.insert("content".to_string(), "Hello world!".into());

    let embeddings_content = vec![content];
    
    let embeddings = embedding_model.embed(embeddings_content).await?;

    println!("Embeddings: {:?}", embeddings);
    

    // -- Completion Model --
    let completion_model = CompletionModel::new("text-bison", None)?;

    let prompts = vec!["Hello world!".to_string()];

    let mut completion_stream = completion_model.stream_complete(prompts, 10, 0.5).await?;

    while let Some(completion_response) = completion_stream.next().await {
        println!("Model Response: {:?}", completion_response);
    }

    Ok(())
}
