use serde_json::Value;
use std::collections::HashMap;

use crate::error::ApiError;

use super::{
    impls::replit_chat_model::ReplitChatModel,
    structs::{ChatModelResponse, ChatSession},
};

pub enum ChatModelInner {
    ReplitChat(ReplitChatModel),
    // Add other models here in the future if needed.
}

pub struct ChatModel {
    inner: ChatModelInner,
}

impl ChatModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, ApiError> {
        let inner = match model_name {
            "chat-bison" => {
                ChatModelInner::ReplitChat(ReplitChatModel::new("chat-bison", server_url)?)
            }
            _ => {
                return Err(ApiError::ModelCreationError(
                    "No matching chat model".to_string(),
                ))
            }
        };

        Ok(Self { inner })
    }

    pub async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, ApiError> {
        match &self.inner {
            ChatModelInner::ReplitChat(model) => {
                model.chat(prompts, max_output_tokens, temperature).await
            } // Handle other models similarly if added in the future.
        }
    }

    pub async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> impl futures_util::stream::Stream<Item = Result<ChatModelResponse, ApiError>> {
        match &self.inner {
            ChatModelInner::ReplitChat(model) => {
                model
                    .stream_chat(prompts, max_output_tokens, temperature)
                    .await
            }
        }
    }

    pub fn build_request_payload(
        &self,
        prompts: &[ChatSession],
        max_output_tokens: i32,
        temperature: f32,
    ) -> HashMap<String, Value> {
        match &self.inner {
            ChatModelInner::ReplitChat(model) => {
                model.build_request_payload(prompts, max_output_tokens, temperature)
            } // Handle other models similarly if added in the future.
        }
    }
}
