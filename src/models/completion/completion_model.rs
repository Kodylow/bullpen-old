use serde_json::Value;
use std::collections::HashMap;

use crate::{error::ApiError, models::base::Model};

use super::{
    impls::replit_completion_model::ReplitCompletionModel, structs::CompletionModelResponse,
    CompletionModels,
};

pub enum CompletionModelInner {
    ReplitCompletion(ReplitCompletionModel),
}

pub struct CompletionModel {
    inner: CompletionModelInner,
}

impl CompletionModel {
    pub fn new(model_name: CompletionModels, server_url: Option<&str>) -> Result<Self, ApiError> {
        let inner = match model_name {
            CompletionModels::TextBison => CompletionModelInner::ReplitCompletion(
                ReplitCompletionModel::new("text-bison", server_url)?,
            ),
            _ => {
                return Err(ApiError::ModelCreationError(
                    "No matching completion model".to_string(),
                ))
            }
        };

        Ok(Self { inner })
    }

    pub async fn complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<CompletionModelResponse, ApiError> {
        match &self.inner {
            CompletionModelInner::ReplitCompletion(model) => {
                model
                    .complete(prompts, max_output_tokens, temperature)
                    .await
            }
        }
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
        match &self.inner {
            CompletionModelInner::ReplitCompletion(model) => {
                model
                    .stream_complete(prompts, max_output_tokens, temperature)
                    .await
            }
        }
    }
}
