use crate::error::ApiError;
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use futures_util::future;
use futures_util::stream;
use futures_util::StreamExt;
use hyper::Body;
use hyper::HeaderMap;
use hyper::Response;
use hyper::StatusCode;
use serde_json::{from_str, Value};
use std::env;
use tokio_stream::Stream;

// Token manager trait and implementations
trait TokenManager {
    fn get_token(&self) -> String;
    fn get_token_type(&self) -> String;
}

struct ReplitIdentityTokenManager;

impl TokenManager for ReplitIdentityTokenManager {
    fn get_token(&self) -> String {
        "REPL_IDENTITY_TOKEN".to_string() // Dummy implementation
    }

    fn get_token_type(&self) -> String {
        "Bearer".to_string()
    }
}

struct L402TokenManager;

impl TokenManager for L402TokenManager {
    fn get_token(&self) -> String {
        env::var("L402_TOKEN").unwrap() // Dummy implementation
    }

    fn get_token_type(&self) -> String {
        "L402".to_string()
    }
}

// Model struct
pub struct Model {
    server_url: String,
    auth: Box<dyn TokenManager>,
}

impl Model {
    pub async fn new(server_url: Option<&str>) -> Result<Self, ApiError> {
        if env::var("REPLIT_DEPLOYMENT").is_ok()
            || env::var("REPL_IDENTITY_KEY").is_ok()
            || env::var("REPL_IDENTITY").is_ok()
            || env::var("REPL_ID").is_ok()
        {
            Ok(Self {
                server_url: server_url.map_or_else(|| "rootUrl".to_string(), ToString::to_string),
                auth: Box::new(ReplitIdentityTokenManager),
            })
        } else {
            Ok(Self {
                server_url: server_url
                    .map_or_else(|| "matadorUrl".to_string(), ToString::to_string),
                auth: Box::new(L402TokenManager),
            })
        }
    }

    async fn check_response(&self, response: Response<Body>) -> Result<(), ApiError> {
        let status = response.status();
        let bytes = hyper::body::to_bytes(response.into_body()).await?;
        let body: Value = serde_json::from_slice(&bytes)?;

        if status == StatusCode::BAD_REQUEST {
            return Err(ApiError::BadRequest(
                body["detail"].as_str().unwrap_or("").to_string(),
            ));
        }

        if status != StatusCode::OK {
            return Err(ApiError::InvalidResponse(
                body["detail"].as_str().unwrap_or("").to_string(),
            ));
        }

        Ok(())
    }

    fn get_auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("{} {}", self.auth.get_token_type(), self.auth.get_token())
                .parse()
                .unwrap(),
        );
        headers
    }

    async fn parse_streaming_response(
        &self,
        response: Response<Body>,
    ) -> impl Stream<Item = Result<Value, ApiError>> {
        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let stream = stream::iter(vec![Ok(bytes)]);

        let mut buffer = BytesMut::new();

        stream.map(move |chunk: Result<bytes::Bytes, std::io::Error>| {
            let new_chunk = chunk.unwrap();
            buffer.put(new_chunk);

            let s = std::str::from_utf8(&buffer).unwrap();
            let mut start_idx = 0;

            loop {
                match from_str::<Value>(&s[start_idx..]) {
                    Ok(value) => {
                        start_idx += value.to_string().len();
                        return Ok(value);
                    }
                    Err(_) => break,
                }
            }

            buffer.advance(start_idx);

            Err(ApiError::InvalidResponse("Invalid JSON".to_string()))
        })
    }
}

// Remaining methods need more context to be translated properly, but this should give you a good starting point.
