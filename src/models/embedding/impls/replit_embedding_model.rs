use std::collections::HashMap;

use serde_json::Value;

use crate::error::ApiError;
use crate::models::base::Model;
use crate::models::embedding::embedding_model::EmbeddingModelTrait;
use crate::models::embedding::structs::EmbeddingModelResponse;

pub struct ReplitEmbeddingModel {
    base: Model,
    model_name: String,
}

impl ReplitEmbeddingModel {
    pub fn new(model_name: &str, server_url: Option<&str>) -> Result<Self, ApiError> {
        let base = Model::new(server_url)?;
        let model_name = model_name.to_string();
        Ok(ReplitEmbeddingModel { base, model_name })
    }

    fn build_request_payload(
        &self,
        content: &Vec<String>,
        parameters: &HashMap<String, Value>,
    ) -> HashMap<String, Value> {
        let mut payload = HashMap::new();
        payload.insert("model".to_string(), self.model_name.clone().into());

        let content: Vec<HashMap<String, String>> = content
            .into_iter()
            .map(|text| {
                let mut map = HashMap::new();
                map.insert("content".to_string(), text.clone());
                map
            })
            .collect();

        let mut params: HashMap<String, Value> = parameters.clone();
        params.insert(
            "content".to_string(),
            serde_json::to_value(content).unwrap(),
        );

        payload.insert(
            "parameters".to_string(),
            serde_json::to_value(params).unwrap(),
        );

        payload
    }
}

#[async_trait::async_trait(?Send)]
impl EmbeddingModelTrait for ReplitEmbeddingModel {
    async fn embed(&self, content: Vec<String>) -> Result<EmbeddingModelResponse, ApiError> {
        let payload = self.build_request_payload(&content, &HashMap::new());

        let req = self
            .base
            .client
            .post(&format!("{}/v1beta/embedding", &self.base.server_url))
            .json(&payload)
            .build()?;

        let res = self.base.client.execute(req).await?;

        self.base.check_response(&res)?;

        // Parse the bytes into an EmbeddingModelResponse
        let embedding_response: EmbeddingModelResponse =
            serde_json::from_slice(&res.bytes().await?)?;

        Ok(embedding_response)
    }
}
