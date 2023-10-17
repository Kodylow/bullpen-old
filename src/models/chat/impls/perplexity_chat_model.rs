use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};

use crate::models::base::structs::{Metadata, PinBoxStream, TokenCountMetadata};
use crate::models::base::Model;
use crate::models::chat::chat_model::ChatModelTrait;
use crate::models::chat::structs::{
    Candidate, ChatModelResponse, ChatPromptResponse, ChatSession, Role,
};
use crate::models::ChatMessage;

pub struct PerplexityChatModel {
    base: Model,
    model_name: String,
}

impl PerplexityChatModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, anyhow::Error> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(PerplexityChatModel { base, model_name })
    }

    pub fn build_request_payload(
        &self,
        prompts: &[ChatSession],
        max_output_tokens: i32,
        temperature: f32,
        stream: bool,
    ) -> Result<PerplexityChatCompletionParameters, anyhow::Error> {
        let mut messages = vec![];
        for prompt in prompts {
            // Convert ChatExample into PerplexityChatMessage
            for example in &prompt.examples {
                messages.push(PerplexityChatMessage {
                    role: "user".to_string(), // Input is always from User
                    content: example.input.content.clone(),
                });
                messages.push(PerplexityChatMessage {
                    role: "assistant".to_string(), // Output is always from Assistant
                    content: example.output.content.clone(),
                });
            }
            // Convert ChatMessage into PerplexityChatMessage
            for message in &prompt.messages {
                messages.push(PerplexityChatMessage {
                    role: message.author.clone().to_string(),
                    content: message.content.clone(),
                });
            }
        }

        let parameters = PerplexityChatCompletionParameters {
            model: self.model_name.clone(),
            messages,
            temperature,
            max_tokens: Some(max_output_tokens as u32),
            stream: Some(stream),
        };

        Ok(parameters)
    }
}

#[async_trait::async_trait(?Send)]
impl ChatModelTrait for PerplexityChatModel {
    async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, anyhow::Error> {
        let payload =
            self.build_request_payload(&prompts, max_output_tokens, temperature, false)?;

        let req = self
            .base
            .client // Use the client from base
            .post(&format!(
                "{}/perplexity/chat/completions",
                &self.base.server_url
            ))
            .json(&payload)
            .build()?;

        let mut res = self.base.client.execute(req).await?; // Use the client from base

        self.base.check_response(&mut res)?;

        // Parse the bytes into a PerplexityChatCompletionResponse
        let perplexity_response: PerplexityChatCompletionResponse =
            serde_json::from_slice(&res.bytes().await?)?;

        // Convert to ChatModelResponse
        let chat_response: ChatModelResponse =
            perplexity_response_to_chat_model_response(perplexity_response)?;

        Ok(chat_response)
    }

    async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> PinBoxStream<ChatModelResponse> {
        let payload_result =
            self.build_request_payload(&prompts, max_output_tokens, temperature, true);

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
            .post(&format!(
                "{}/perplexity/chat/completions",
                &self.base.server_url
            ))
            .json(&payload)
            .build()
            .unwrap();

        let res = self.base.client.execute_stream(req).await;
        Box::pin(res.map(|res| {
            let res = res?;
            let perplexity_response: PerplexityChatCompletionResponse =
                serde_json::from_slice(&res)?;
            let chat_response = perplexity_response_to_chat_model_response(perplexity_response)?;

            Ok(chat_response)
        }))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerplexityChatCompletionParameters {
    pub model: String,
    pub messages: Vec<PerplexityChatMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StopToken {
    String(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerplexityChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerplexityChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u32,
    pub model: String,
    pub choices: Vec<PerplexityChatCompletionChoice>,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerplexityChatCompletionChoice {
    pub index: u32,
    pub message: PerplexityChatMessage,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerplexityAIChatMessage {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FinishReason {
    #[serde(rename(deserialize = "stop"))]
    StopSequenceReached,
    #[serde(rename(deserialize = "length"))]
    TokenLimitReached,
    #[serde(rename(deserialize = "content_filter"))]
    ContentFilterFlagged,
}

fn perplexity_response_to_chat_model_response(
    perplexity_res: PerplexityChatCompletionResponse,
) -> anyhow::Result<ChatModelResponse> {
    let responses = perplexity_res
        .choices
        .into_iter()
        .map(|choice| {
            let message = ChatMessage {
                content: choice.message.content,
                author: choice.message.role,
            };
            let candidate = Candidate {
                message,
                metadata: None, // This field doesn't exist in PerplexityChatCompletionChoice
            };
            ChatPromptResponse {
                candidates: vec![candidate],
            }
        })
        .collect();

    let metadata = Metadata {
        input_token_count: Some(TokenCountMetadata {
            billable_tokens: perplexity_res.usage.prompt_tokens as i32,
            unbilled_tokens: 0,
            billable_characters: 0,
            unbilled_characters: 0,
        }),
        output_token_count: Some(TokenCountMetadata {
            billable_tokens: perplexity_res.usage.completion_tokens as i32,
            unbilled_tokens: 0,
            billable_characters: 0,
            unbilled_characters: 0,
        }),
    };

    let chat_model_response = ChatModelResponse {
        metadata: Some(metadata),
        responses,
    };

    Ok(chat_model_response)
}
