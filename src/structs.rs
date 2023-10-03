use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenCountMetadata {
    pub billable_tokens: i32,
    pub unbilled_tokens: i32,
    pub billable_characters: i32,
    pub unbilled_characters: i32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub input_token_count: Option<TokenCountMetadata>,
    pub output_token_count: Option<TokenCountMetadata>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatExample {
    pub input: ChatMessage,
    pub output: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub context: String,
    pub examples: Vec<ChatExample>,
    pub messages: Vec<ChatMessage>,
}

impl ChatSession {
    pub fn model_dump(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("context".to_string(), serde_json::json!(self.context));
        map.insert("examples".to_string(), serde_json::json!(self.examples));
        map.insert("messages".to_string(), serde_json::json!(self.messages));
        serde_json::Value::Object(map)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatModelRequest {
    pub model: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub message: ChatMessage,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatPromptResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatModelResponse {
    pub metadata: Option<Metadata>,
    pub responses: Vec<ChatPromptResponse>,
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
