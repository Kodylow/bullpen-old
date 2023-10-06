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

// pub enum ChatModelInner {
//     ReplitChat(ReplitChatModel),
//     // OpenAiChat(OpenAiChatModel),
// }

// pub struct ChatModel {
//     inner: ChatModelInner,
// }

// impl ChatModel {
//     pub fn new(model_name: ChatModels, server_url: Option<&str>) -> Result<Self, ApiError> {
//         let mut formatted_url = String::new();
//         if let Some(url) = server_url {
//             formatted_url = format!("{}/{}", url, model_name.uri_prefix());
//         }
//         println!("formatted_url: {}", formatted_url);
//         let server_url = if formatted_url.is_empty() {
//             None
//         } else {
//             Some(formatted_url.as_str())
//         };
//         let inner = match model_name {
//             ChatModels::ChatBison => {
//                 ChatModelInner::ReplitChat(ReplitChatModel::new(model_name.as_str(), server_url)?)
//             }
//             // ChatModels::Gpt35Turbo => {
//             //     ChatModelInner::OpenAiChat(OpenAiChatModel::new(model_name.as_str(), server_url)?)
//             // }
//             // ChatModels::Gpt4 => {
//             //     ChatModelInner::OpenAiChat(OpenAiChatModel::new(model_name.as_str(), server_url)?)
//             // }
//             _ => {
//                 return Err(ApiError::ModelCreationError(
//                     "No matching chat model".to_string(),
//                 ))
//             }
//         };

//         Ok(Self { inner })
//     }

//     pub async fn chat(
//         &self,
//         prompts: Vec<ChatSession>,
//         max_output_tokens: i32,
//         temperature: f32,
//     ) -> Result<ChatModelResponse, ApiError> {
//         match &self.inner {
//             ChatModelInner::ReplitChat(model) => {
//                 model.chat(prompts, max_output_tokens, temperature).await
//             } // ChatModelInner::OpenAiChat(model) => {
//               //     model.chat(prompts, max_output_tokens, temperature).await
//               // }
//         }
//     }

//     pub async fn stream_chat(
//         &self,
//         prompts: Vec<ChatSession>,
//         max_output_tokens: i32,
//         temperature: f32,
//     ) -> impl futures_util::stream::Stream<Item = Result<ChatModelResponse, ApiError>> {
//         match &self.inner {
//             ChatModelInner::ReplitChat(model) => {
//                 model
//                     .stream_chat(prompts, max_output_tokens, temperature)
//                     .await
//             } // ChatModelInner::OpenAiChat(model) => {
//               //     model
//               //         .stream_chat(prompts, max_output_tokens, temperature)
//               //         .await
//               // }
//         }
//     }
// }
