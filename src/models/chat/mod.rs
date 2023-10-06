pub mod chat_model;
pub mod impls;
pub mod structs;

pub use self::chat_model::ChatModel;
pub use self::structs::{ChatExample, ChatMessage, ChatSession};
pub use self::ChatModels::ChatBison;

pub enum ChatModels {
    ChatBison,
    Gpt35Turbo,
    Gpt4,
}

impl ChatModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatModels::ChatBison => "chat-bison",
            ChatModels::Gpt35Turbo => "gpt-3.5-turbo",
            ChatModels::Gpt4 => "gpt-4",
        }
    }

    pub fn uri_prefix(&self) -> &str {
        match self {
            ChatModels::ChatBison => "replit",
            ChatModels::Gpt35Turbo => "openai",
            ChatModels::Gpt4 => "openai",
        }
    }
}
