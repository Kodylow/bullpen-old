use futures_util::{stream, StreamExt};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    error::ApiError,
    model::Model,
    structs::{ChatModelResponse, ChatSession},
};

pub struct ChatModel {
    base: Model,
    model_name: String,
}

impl ChatModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, ApiError> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(ChatModel { base, model_name })
    }

    pub async fn chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<ChatModelResponse, ApiError> {
        let client = reqwest::Client::new();
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = client
            .post(&format!("{}/v1beta/chat", &self.base.server_url))
            .json(&payload)
            .build()?;

        let mut res = client.execute(req).await?;

        self.base.check_response(&mut res)?;

        // Parse the bytes into a ChatModelResponse
        let chat_response: ChatModelResponse = serde_json::from_slice(&res.bytes().await?)?;

        Ok(chat_response)
    }

    fn build_request_payload(
        &self,
        prompts: &Vec<ChatSession>,
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

    // Asynchronous streaming method using reqwest
    pub async fn stream_chat(
        &self,
        prompts: Vec<ChatSession>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<
        impl futures_util::stream::Stream<Item = Result<ChatModelResponse, ApiError>>,
        ApiError,
    > {
        let client = Client::new();
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = client
            .post(&format!("{}/v1beta/chat", &self.base.server_url))
            .json(&payload)
            .build()?;

        let res = client.execute(req).await?;

        self.base.check_response(&res)?;

        let chunks_stream = res.bytes_stream().map(|result_chunk| match result_chunk {
            Ok(chunk) => {
                let chat_response: Result<ChatModelResponse, _> = serde_json::from_slice(&chunk);
                chat_response.map_err(|e| ApiError::SerdeError(e))
            }
            Err(e) => Err(ApiError::ReqwestError(e)),
        });

        Ok(chunks_stream)
    }
}
