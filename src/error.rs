use serde_json::Error as JsonError;

// Error handling
#[derive(Debug)]
pub enum ApiError {
    InvalidResponse(String),
    BadRequest(String),
}

impl From<JsonError> for ApiError {
    fn from(err: JsonError) -> Self {
        ApiError::InvalidResponse(err.to_string())
    }
}

impl From<hyper::Error> for ApiError {
    fn from(err: hyper::Error) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}
