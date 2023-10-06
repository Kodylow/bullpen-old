pub mod embedding_model;
pub mod impls;
pub mod structs;
pub use self::embedding_model::EmbeddingModel;
pub use self::EmbeddingModels::TextEmbeddingGecko;

pub enum EmbeddingModels {
    TextEmbeddingGecko,
}

impl EmbeddingModels {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmbeddingModels::TextEmbeddingGecko => "textembedding-gecko",
        }
    }
}
