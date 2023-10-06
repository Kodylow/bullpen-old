use std::collections::HashMap;

use futures_util::stream::{self, StreamExt};
use log::warn;
use serde_json::Value;

use crate::{
    error::ApiError,
    models::{
        base::Model,
        chat::structs::{ChatModelResponse, ChatSession},
    },
};

pub struct OpenAiChatModel {
    base: Model,
    model_name: String,
}

impl OpenAiChatModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, ApiError> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(OpenAiChatModel { base, model_name })
    }

    pub async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, ApiError> {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

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

    pub async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> impl futures_util::stream::Stream<Item = Result<ChatModelResponse, ApiError>> {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = self
            .base
            .client // Use the client from base
            .post(&format!("{}/v1beta/chat_streaming", &self.base.server_url))
            .json(&payload)
            .build()
            .unwrap();

        let res = self.base.client.execute_stream(req).await; // Use the client from base

        res.map(|res| {
            let res = res?;
            let chat_response: ChatModelResponse = serde_json::from_slice(&res)?;
            Ok(chat_response)
        })
    }

    pub fn build_request_payload(
        &self,
        prompts: &[ChatSession],
        max_output_tokens: i32,
        temperature: f32,
    ) -> HashMap<String, Value> {
        let mut payload = HashMap::new();
        payload.insert("model".to_string(), self.model_name.clone().into());

        let mut parameters: HashMap<String, serde_json::Value> = HashMap::new();
        parameters.insert(
            "prompts".to_string(),
            prompts
                .iter()
                .map(|p| p.model_dump().clone())
                .collect::<Vec<_>>()
                .into(),
        );
        parameters.insert("temperature".to_string(), temperature.into());
        parameters.insert("maxOutputTokens".to_string(), max_output_tokens.into());

        payload.insert(
            "parameters".to_string(),
            serde_json::to_value(parameters).unwrap(),
        );

        payload
    }
}
