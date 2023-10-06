use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::base::structs::TokenCountMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub enum EmbeddingModel {
    TextEmbeddingGecko,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Embedding {
    pub values: Vec<f64>,
    pub token_count_metadata: Option<TokenCountMetadata>,
    pub truncated: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingMetadata {
    pub token_count_metadata: Option<TokenCountMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingModelRequest {
    pub model: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingModelResponse {
    pub metadata: Option<EmbeddingMetadata>,
    pub embeddings: Vec<Embedding>,
}
