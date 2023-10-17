use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::base::structs::Metadata;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub author: Role,
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
