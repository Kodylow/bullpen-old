pub mod chat_model;
pub mod impls;
pub mod structs;

pub use self::chat_model::ChatModel;
pub use self::structs::{ChatExample, ChatMessage, ChatSession};
pub use self::ChatModels::ChatBison;

pub enum ChatModels {
    // Replit models
    ChatBison,
    // OpenAI models
    Gpt35Turbo,
    Gpt4,
    // Perplexity models
    Llama2_70bChat,
    Llama2_13bChat,
    Codellama34bInstruct,
    Mistral7bInstruct,
}

impl ChatModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatModels::ChatBison => "chat-bison",
            ChatModels::Gpt35Turbo => "gpt-3.5-turbo",
            ChatModels::Gpt4 => "gpt-4",
            ChatModels::Llama2_70bChat => "llama-2-70b-chat",
            ChatModels::Llama2_13bChat => "llama-2-13b-chat",
            ChatModels::Codellama34bInstruct => "codellama-34b-instruct",
            ChatModels::Mistral7bInstruct => "mistral-7b-instruct",
        }
    }

    pub fn uri_prefix(&self) -> &str {
        match self {
            ChatModels::ChatBison => "replit",
            ChatModels::Gpt35Turbo => "openai",
            ChatModels::Gpt4 => "openai",
            ChatModels::Llama2_70bChat => "perplexity",
            ChatModels::Llama2_13bChat => "perplexity",
            ChatModels::Codellama34bInstruct => "perplexity",
            ChatModels::Mistral7bInstruct => "perplexity",
        }
    }
}
