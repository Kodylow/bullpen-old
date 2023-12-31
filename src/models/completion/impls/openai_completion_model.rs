use std::collections::HashMap;

use futures_util::stream::StreamExt;
use log::info;
use serde_json::Value;

use crate::models::base::structs::PinBoxStream;
use crate::models::base::Model;
use crate::models::completion::completion_model::CompletionModelTrait;
use crate::models::completion::structs::CompletionModelResponse;

pub struct OpenAICompletionModel {
    base: Model,
    model_name: String,
}

impl OpenAICompletionModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, anyhow::Error> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(OpenAICompletionModel { base, model_name })
    }

    pub fn build_request_payload(
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

#[async_trait::async_trait(?Send)]
impl CompletionModelTrait for OpenAICompletionModel {
    async fn complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<CompletionModelResponse, anyhow::Error> {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = self
            .base
            .client
            .post(&format!("{}/openai/v1/completion", &self.base.server_url))
            .json(&payload)
            .build()?;

        let res = self.base.client.execute(req).await?;

        info!("Response: {:?}", res);

        self.base.check_response(&res)?;

        let completion_response: CompletionModelResponse =
            serde_json::from_slice(&res.bytes().await?)?;

        Ok(completion_response)
    }

    async fn stream_complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> PinBoxStream<CompletionModelResponse> {
        let payload = self.build_request_payload(&prompts, max_output_tokens, temperature);

        let req = self
            .base
            .client
            .post(&format!("{}/v1beta/completion", &self.base.server_url))
            .json(&payload)
            .build()
            .unwrap();

        let res = self.base.client.execute(req).await.unwrap().bytes_stream();

        Box::pin(res.map(|res| {
            let res = res?;
            let chat_response: CompletionModelResponse = serde_json::from_slice(&res)?;
            Ok(chat_response)
        }))
    }
}
