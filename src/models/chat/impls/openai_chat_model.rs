use std::collections::HashMap;

use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::base::structs::PinBoxStream;
use crate::models::base::Model;
use crate::models::chat::chat_model::ChatModelTrait;
use crate::models::chat::structs::{ChatModelRequest, ChatModelResponse, ChatSession, Role};

pub struct OpenAiChatModel {
    base: Model,
    model_name: String,
}

impl OpenAiChatModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, anyhow::Error> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(OpenAiChatModel { base, model_name })
    }

    pub fn build_request_payload(
        &self,
        prompts: &[ChatSession],
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<OpenAIChatCompletionParameters, anyhow::Error> {
        let mut messages = vec![];
        for prompt in prompts {
            // Convert ChatExample into OpenAIChatMessage
            for example in &prompt.examples {
                messages.push(OpenAIChatMessage {
                    role: Role::User, // Input is always from User
                    content: example.input.content.clone(),
                });
                messages.push(OpenAIChatMessage {
                    role: Role::Assistant, // Output is always from Assistant
                    content: example.output.content.clone(),
                });
            }
            // Convert ChatMessage into OpenAIChatMessage
            for message in &prompt.messages {
                messages.push(OpenAIChatMessage {
                    role: message.author.clone(),
                    content: message.content.clone(),
                });
            }
        }

        let parameters = OpenAIChatCompletionParameters {
            model: self.model_name.clone(),
            messages,
            temperature,
            max_tokens: Some(max_output_tokens as u32),
        };

        Ok(parameters)
    }
}

#[async_trait::async_trait(?Send)]
impl ChatModelTrait for OpenAiChatModel {
    async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, anyhow::Error> {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature)?;

        let req = self
            .base
            .client // Use the client from base
            .post(&format!("{}/v1beta/chat", &self.base.server_url))
            .json(&payload)
            .build()?;

        let mut res = self.base.client.execute(req).await?; // Use the client from base

        self.base.check_response(&mut res)?;

        // Parse the bytes into a ChatModelResponse
        let chat_response: ChatModelResponse = serde_json::from_slice(&res.bytes().await?)?;

        Ok(chat_response)
    }

    async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> PinBoxStream<ChatModelResponse> {
        let payload_result = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let payload = match payload_result {
            Ok(p) => p,
            Err(e) => {
                return Box::pin(stream::once(async move {
                    Err(anyhow::anyhow!("Failed to build request payload: {}", e))
                }))
            }
        };

        let req = self
            .base
            .client // Use the client from base
            .post(&format!("{}/v1beta/chat_streaming", &self.base.server_url))
            .json(&payload)
            .build()
            .unwrap();

        let res = self.base.client.execute_stream(req).await; // Use the client from base

        Box::pin(res.map(|res| {
            let res = res?;
            let chat_response: ChatModelResponse = serde_json::from_slice(&res)?;
            Ok(chat_response)
        }))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIChatCompletionParameters {
    pub model: String,
    pub messages: Vec<OpenAIChatMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StopToken {
    String(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIChatMessage {
    pub role: Role,
    pub content: String,
}
