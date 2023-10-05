use std::fmt;

use serde_json::Error as JsonError;

// Error handling
pub enum ApiError {
    ReqwestError(reqwest::Error),
    SerdeError(serde_json::Error),
    InvalidRequest(String),
    ModelCreationError(String),
}

impl From<JsonError> for ApiError {
    fn from(err: JsonError) -> Self {
        ApiError::SerdeError(err)
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::ReqwestError(err)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::ReqwestError(err) => write!(f, "Reqwest error {}", err),
            ApiError::SerdeError(err) => write!(f, "Serde error {}", err),
            ApiError::InvalidRequest(err) => write!(f, "Invalid request {}", err),
            ApiError::ModelCreationError(err) => write!(f, "Model Creation Error {}", err),
        }
    }
}

impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ApiError {}
