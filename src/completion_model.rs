use crate::{error::ApiError, model::Model, structs::CompletionModelResponse};
use futures_util::stream;
use serde_json::Value;
use std::collections::HashMap;

pub struct CompletionModel {
    base: Model,
    model_name: String,
}

impl CompletionModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, ApiError> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(CompletionModel { base, model_name })
    }

    pub async fn complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<CompletionModelResponse, ApiError> {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = self
            .base
            .client
            .post(&format!("{}/v1beta/completion", &self.base.server_url))
            .json(&payload)
            .build()?;

        let mut res = self.base.client.execute(req).await?;

        self.base.check_response(&res)?;

        // Parse the bytes into a CompletionModelResponse
        let completion_response: CompletionModelResponse =
            serde_json::from_slice(&res.bytes().await?)?;

        Ok(completion_response)
    }

    pub async fn stream_complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<
        impl futures_util::stream::Stream<Item = Result<CompletionModelResponse, ApiError>>,
        ApiError,
    > {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = self
            .base
            .client
            .post(&format!("{}/v1beta/completion", &self.base.server_url))
            .json(&payload)
            .build()?;

        let res = self.base.client.execute(req).await?;

        self.base.check_response(&res)?;

        // Parse the bytes into a CompletionModelResponse
        let completion_response: CompletionModelResponse =
            serde_json::from_slice(&res.bytes().await?)?;

        let stream = stream::iter(vec![Ok(completion_response)]);

        Ok(stream)
    }

    fn build_request_payload(
        &self,
        prompts: &Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> HashMap<String, Value> {
        let mut payload = HashMap::new();
        payload.insert("model".to_string(), self.model_name.clone().into());

        let mut parameters: HashMap<String, serde_json::Value> = HashMap::new();
        parameters.insert("prompts".to_string(), prompts.clone().into());
        parameters.insert("temperature".to_string(), temperature.into());
        parameters.insert("maxOutputTokens".to_string(), max_output_tokens.into());

        payload.insert(
            "parameters".to_string(),
            serde_json::to_value(parameters).unwrap(),
        );

        payload
    }
}
