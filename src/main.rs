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
        context: "You are a philosophy bot".to_string(),
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
            content: "What is the meaning of life?".to_string(),
            author: "USER".to_string(),
        }],
    };
    let mut chat_stream = chat_model.stream_chat(vec![chat_session], 10, 0.7).await?;
    panic!("This is a panic");
    let mut counter = 0;
    let max_responses = 100; // Set this to the maximum number of responses you want to process

    while let Some(chat_response) = chat_stream.next().await {
        match chat_response {
            Ok(response) => {
                println!("{:?}", response);
                counter += 1;
                if counter >= max_responses {
                    break;
                }
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
