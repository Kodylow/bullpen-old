pub mod chat_model;
pub mod impls;
pub mod structs;

pub use self::chat_model::ChatModel;
pub use self::structs::{ChatExample, ChatMessage, ChatSession};
pub use self::ChatModels::ChatBison;

pub enum ChatModels {
    ChatBison,
}

impl ChatModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatModels::ChatBison => "chat-bison",
        }
    }
}
