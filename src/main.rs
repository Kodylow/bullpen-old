use bullpen::{
    structs::{ChatExample, ChatMessage, ChatSession},
    ChatModel,
};
use std::error::Error;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    // let mut chat_stream = chat_model.stream_chat(vec![chat_session], 10, 0.5).await?;

    // while let Some(chat_response) = chat_stream.next().await {
    //     println!("Model Response: {:?}", chat_response);
    // }

    let chat_response = chat_model.chat(vec![chat_session], 10, 0.5).await.unwrap();

    println!("Model Response: {:?}", chat_response);

    Ok(())
}
