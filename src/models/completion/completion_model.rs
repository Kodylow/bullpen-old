use super::impls::replit_completion_model::ReplitCompletionModel;
use super::structs::CompletionModelResponse;
use super::CompletionModels;
use crate::error::ApiError;
use crate::models::base::structs::PinBoxStream;

pub struct CompletionModel {
    inner: Box<dyn CompletionModelTrait>,
}

impl CompletionModel {
    pub fn new(model_name: CompletionModels, server_url: Option<&str>) -> Result<Self, ApiError> {
        match model_name {
            CompletionModels::TextBison => Ok(Self {
                inner: Box::new(ReplitCompletionModel::new(model_name.as_str(), server_url)?),
            }),
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait CompletionModelTrait {
    async fn complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<CompletionModelResponse, ApiError>;
    async fn stream_complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> PinBoxStream<CompletionModelResponse>;
}

#[async_trait::async_trait(?Send)]
impl CompletionModelTrait for CompletionModel {
    async fn complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> Result<CompletionModelResponse, ApiError> {
        self.inner
            .complete(prompts, max_output_tokens, temperature)
            .await
    }
    async fn stream_complete(
        &self,
        prompts: Vec<String>,
        max_output_tokens: i32,
        temperature: f32,
    ) -> PinBoxStream<CompletionModelResponse> {
        self.inner
            .stream_complete(prompts, max_output_tokens, temperature)
            .await
    }
}
