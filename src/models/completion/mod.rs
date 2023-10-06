pub mod completion_model;
pub mod impls;
pub mod structs;
pub use self::completion_model::CompletionModel;
pub use self::CompletionModels::TextBison;

pub enum CompletionModels {
    TextBison,
}

impl CompletionModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompletionModels::TextBison => "text-bison",
        }
    }
}
