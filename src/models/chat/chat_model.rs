use serde_json::Value;
use std::{collections::HashMap, pin::Pin};

use crate::{error::ApiError, models::base::structs::PinBoxStream};

use super::{
    impls::{OpenAiChatModel, ReplitChatModel},
    structs::{ChatModelResponse, ChatSession},
    ChatModels,
};

pub struct ChatModel {
    inner: Box<dyn ChatModelTrait>,
}

impl ChatModel {
    pub fn new(model_name: ChatModels, server_url: Option<&str>) -> Result<Self, ApiError> {
        match model_name {
            ChatModels::ChatBison => Ok(Self {
                inner: Box::new(ReplitChatModel::new(model_name.as_str(), server_url)?),
            }),
            ChatModels::Gpt35Turbo => Ok(Self {
                inner: Box::new(OpenAiChatModel::new(model_name.as_str(), server_url)?),
            }),
            ChatModels::Gpt4 => Ok(Self {
                inner: Box::new(OpenAiChatModel::new(model_name.as_str(), server_url)?),
            }),
            _ => Err(ApiError::ModelCreationError(
                "No matching chat model".to_string(),
            )),
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait ChatModelTrait {
    async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, ApiError>;
    async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Pin<Box<dyn futures_util::stream::Stream<Item = Result<ChatModelResponse, ApiError>>>>;
}

#[async_trait::async_trait(?Send)]
impl ChatModelTrait for ChatModel {
    async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, ApiError> {
        self.inner
            .chat(prompts, max_output_tokens, temperature)
            .await
    }
    async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> PinBoxStream<ChatModelResponse> {
        self.inner
            .stream_chat(prompts, max_output_tokens, temperature)
            .await
    }
}
