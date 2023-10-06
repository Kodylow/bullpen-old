use serde_json::Value;
use std::collections::HashMap;

use crate::error::ApiError;

use super::{
    impls::{OpenAiChatModel, ReplitChatModel},
    structs::{ChatModelResponse, ChatSession},
    ChatModels,
};

pub enum ChatModelInner {
    ReplitChat(ReplitChatModel),
    OpenAiChat(OpenAiChatModel),
}

pub struct ChatModel {
    inner: ChatModelInner,
}

impl ChatModel {
    pub fn new(model_name: ChatModels, mut server_url: Option<&str>) -> Result<Self, ApiError> {
        if server_url.is_some() {
            server_url =
                Some(format!("{}/{}", server_url.unwrap(), model_name.uri_prefix()).as_str());
        }
        let inner = match model_name {
            ChatModels::ChatBison => {
                ChatModelInner::ReplitChat(ReplitChatModel::new(model_name.as_str(), server_url)?)
            }
            ChatModels::Gpt35Turbo => {
                ChatModelInner::OpenAiChat(OpenAiChatModel::new(model_name.as_str(), server_url)?)
            }
            ChatModels::Gpt4 => {
                ChatModelInner::OpenAiChat(OpenAiChatModel::new(model_name.as_str(), server_url)?)
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
            }
            ChatModelInner::OpenAiChat(model) => {
                model.chat(prompts, max_output_tokens, temperature).await
            }
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
            ChatModelInner::OpenAiChat(model) => {
                model
                    .stream_chat(prompts, max_output_tokens, temperature)
                    .await
            }
        }
    }
}
