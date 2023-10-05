use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::base::structs::Metadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionModelRequest {
    pub model: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionModelResponse {
    pub metadata: Option<Metadata>,
    pub responses: Vec<PromptResponse>,
}
