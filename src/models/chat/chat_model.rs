use std::pin::Pin;

use super::impls::{OpenAiChatModel, PerplexityChatModel, ReplitChatModel};
use super::structs::{ChatModelResponse, ChatSession};
use super::ChatModels;
use crate::models::base::structs::PinBoxStream;

pub struct ChatModel {
    inner: Box<dyn ChatModelTrait>,
}

impl ChatModel {
    pub fn new(model_name: ChatModels, server_url: Option<&str>) -> Result<Self, anyhow::Error> {
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
            ChatModels::Llama2_70bChat => Ok(Self {
                inner: Box::new(PerplexityChatModel::new(model_name.as_str(), server_url)?),
            }),
            ChatModels::Llama2_13bChat => Ok(Self {
                inner: Box::new(PerplexityChatModel::new(model_name.as_str(), server_url)?),
            }),
            ChatModels::Codellama34bInstruct => Ok(Self {
                inner: Box::new(PerplexityChatModel::new(model_name.as_str(), server_url)?),
            }),
            ChatModels::Mistral7bInstruct => Ok(Self {
                inner: Box::new(PerplexityChatModel::new(model_name.as_str(), server_url)?),
            }),
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
    ) -> Result<ChatModelResponse, anyhow::Error>;
    async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Pin<Box<dyn futures_util::stream::Stream<Item = Result<ChatModelResponse, anyhow::Error>>>>;
}

#[async_trait::async_trait(?Send)]
impl ChatModelTrait for ChatModel {
    async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, anyhow::Error> {
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
