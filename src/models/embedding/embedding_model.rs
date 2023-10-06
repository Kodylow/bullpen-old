use crate::{error::ApiError, models::base::Model};

use serde_json::Value;
use std::collections::HashMap;

use super::{
    impls::replit_embedding_model::ReplitEmbeddingModel, structs::EmbeddingModelResponse,
    EmbeddingModels,
};

pub enum EmbeddingModelInner {
    ReplitEmbedding(ReplitEmbeddingModel),
}

pub struct EmbeddingModel {
    inner: EmbeddingModelInner,
}

impl EmbeddingModel {
    pub fn new(model_name: EmbeddingModels, server_url: Option<&str>) -> Result<Self, ApiError> {
        let inner = match model_name {
            EmbeddingModels::TextEmbeddingGecko => EmbeddingModelInner::ReplitEmbedding(
                ReplitEmbeddingModel::new("text-bison", server_url)?,
            ),
            _ => {
                return Err(ApiError::ModelCreationError(
                    "No matching completion model".to_string(),
                ))
            }
        };

        Ok(Self { inner })
    }

    pub async fn embed(&self, content: Vec<String>) -> Result<EmbeddingModelResponse, ApiError> {
        match &self.inner {
            EmbeddingModelInner::ReplitEmbedding(model) => model.embed(content).await,
        }
    }
}
