pub mod completion_model;
pub mod impls;
pub mod structs;
pub use self::completion_model::CompletionModel;
pub use self::CompletionModels::TextBison;

pub enum CompletionModels {
    TextBison,
    Gpt35Turbo,
    Gpt4,
}

impl CompletionModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompletionModels::TextBison => "text-bison",
            CompletionModels::Gpt35Turbo => "gpt-3.5-turbo",
            CompletionModels::Gpt4 => "gpt-4",
        }
    }
}
