use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};

use crate::models::base::structs::{Metadata, PinBoxStream, TokenCountMetadata};
use crate::models::base::Model;
use crate::models::chat::chat_model::ChatModelTrait;
use crate::models::chat::structs::{
    Candidate, ChatModelResponse, ChatPromptResponse, ChatSession, Role,
};
use crate::models::ChatMessage;

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
        stream: bool,
    ) -> Result<OpenAIChatCompletionParameters, anyhow::Error> {
        let mut messages = vec![];
        for prompt in prompts {
            // Convert ChatExample into OpenAIChatMessage
            for example in &prompt.examples {
                messages.push(OpenAIChatMessage {
                    role: "user".to_string(), // Input is always from User
                    content: example.input.content.clone(),
                });
                messages.push(OpenAIChatMessage {
                    role: "assistant".to_string(), // Output is always from Assistant
                    content: example.output.content.clone(),
                });
            }
            // Convert ChatMessage into OpenAIChatMessage
            for message in &prompt.messages {
                messages.push(OpenAIChatMessage {
                    role: message.author.clone().to_string(),
                    content: message.content.clone(),
                });
            }
        }

        let parameters = OpenAIChatCompletionParameters {
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
impl ChatModelTrait for OpenAiChatModel {
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
                "{}/openai/v1/chat/completions",
                &self.base.server_url
            ))
            .json(&payload)
            .build()?;

        let mut res = self.base.client.execute(req).await?; // Use the client from base

        self.base.check_response(&mut res)?;

        // Parse the bytes into a OpenaiChatCompletionResponse
        let openai_response: OpenAIChatCompletionResponse =
            serde_json::from_slice(&res.bytes().await?)?;

        // Convert to ChatModelResponse
        let chat_response: ChatModelResponse =
            openai_response_to_chat_model_response(openai_response)?;

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
                "{}/openai/v1/chat/completions",
                &self.base.server_url
            ))
            .json(&payload)
            .build()
            .unwrap();

        let res = self.base.client.execute_stream(req).await;
        Box::pin(res.map(|res| {
            let res = res?;
            let openai_response: OpenAIChatCompletionResponse = serde_json::from_slice(&res)?;
            let chat_response = openai_response_to_chat_model_response(openai_response)?;

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
    pub stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StopToken {
    String(String),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAIChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u32,
    pub model: String,
    pub choices: Vec<OpenAIChatCompletionChoice>,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAIChatCompletionChoice {
    pub index: u32,
    pub message: OpenAIChatMessage,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenaiAIChatMessage {
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

fn openai_response_to_chat_model_response(
    openai_res: OpenAIChatCompletionResponse,
) -> anyhow::Result<ChatModelResponse> {
    let responses = openai_res
        .choices
        .into_iter()
        .map(|choice| {
            let message = ChatMessage {
                content: choice.message.content,
                author: choice.message.role,
            };
            let candidate = Candidate {
                message,
                metadata: None, // This field doesn't exist in OpenAIChatCompletionChoice
            };
            ChatPromptResponse {
                candidates: vec![candidate],
            }
        })
        .collect();

    let metadata = Metadata {
        input_token_count: Some(TokenCountMetadata {
            billable_tokens: openai_res.usage.prompt_tokens as i32,
            unbilled_tokens: 0,
            billable_characters: 0,
            unbilled_characters: 0,
        }),
        output_token_count: Some(TokenCountMetadata {
            billable_tokens: openai_res.usage.completion_tokens as i32,
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
