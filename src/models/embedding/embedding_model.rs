use super::impls::replit_embedding_model::ReplitEmbeddingModel;
use super::structs::EmbeddingModelResponse;
use super::EmbeddingModels;
use crate::error::ApiError;

pub enum EmbeddingModelInner {
    ReplitEmbedding(ReplitEmbeddingModel),
}

pub struct EmbeddingModel {
    inner: Box<dyn EmbeddingModelTrait>,
}

impl EmbeddingModel {
    pub fn new(model_name: EmbeddingModels, server_url: Option<&str>) -> Result<Self, ApiError> {
        match model_name {
            EmbeddingModels::TextEmbeddingGecko => Ok(Self {
                inner: Box::new(ReplitEmbeddingModel::new(model_name.as_str(), server_url)?),
            }),
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait EmbeddingModelTrait {
    async fn embed(&self, prompts: Vec<String>) -> Result<EmbeddingModelResponse, ApiError>;
}

#[async_trait::async_trait(?Send)]
impl EmbeddingModelTrait for EmbeddingModel {
    async fn embed(&self, prompts: Vec<String>) -> Result<EmbeddingModelResponse, ApiError> {
        self.inner.embed(prompts).await
    }
}
