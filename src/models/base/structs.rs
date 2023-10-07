use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::{error::ApiError};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenCountMetadata {
    pub billable_tokens: i32,
    pub unbilled_tokens: i32,
    pub billable_characters: i32,
    pub unbilled_characters: i32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub input_token_count: Option<TokenCountMetadata>,
    pub output_token_count: Option<TokenCountMetadata>,
}

pub type PinBoxStream<T> = Pin<Box<dyn futures_util::stream::Stream<Item = Result<T, ApiError>>>>;
