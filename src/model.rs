use crate::error::ApiError;
use crate::l402_client::L402Client;
use crate::replit_client::ReplitClient;
use crate::token_manager::L402TokenManager;
use crate::token_manager::ReplitIdentityTokenManager;
use crate::token_manager::TokenManager;

use bytes::Bytes;

use futures_util::Stream;

use reqwest::header::HeaderMap;

use reqwest::Client;
use reqwest::Request;
use reqwest::Response;
use reqwest::StatusCode;

use std::env;

pub enum HttpClient {
    ReqwestClient(ReplitClient),
    L402Client(L402Client),
}

impl HttpClient {
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        match self {
            HttpClient::ReqwestClient(client) => client.post(url),
            HttpClient::L402Client(client) => client.post(url),
        }
    }

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        match self {
            HttpClient::ReqwestClient(client) => client.get(url),
            HttpClient::L402Client(client) => client.get(url),
        }
    }

    pub async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        match self {
            HttpClient::ReqwestClient(client) => client.execute(request).await,
            HttpClient::L402Client(client) => client.execute(request).await,
        }
    }

    pub async fn execute_stream(
        &self,
        request: Request,
    ) -> Result<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Unpin>, reqwest::Error> {
        match self {
            HttpClient::ReqwestClient(client) => {
                let res_stream = client.execute(request).await?.bytes_stream();
                Ok(Box::new(res_stream))
            }
            HttpClient::L402Client(client) => {
                let res_stream = client.execute_stream(request).await?;
                Ok(Box::new(res_stream))
            }
        }
    }
}

pub struct Model {
    pub server_url: String,
    pub auth: Box<dyn TokenManager>,
    pub client: HttpClient,
}

impl Model {
    pub fn new(server_url: Option<&str>) -> Result<Self, ApiError> {
        let config = crate::config::get_config();
        if env::var("REPLIT_DEPLOYMENT").is_ok()
            || env::var("REPL_IDENTITY_KEY").is_ok()
            || env::var("REPL_IDENTITY").is_ok()
            || env::var("REPL_ID").is_ok()
        {
            Ok(Self {
                server_url: server_url.map_or_else(|| config.root_url.clone(), ToString::to_string),
                auth: Box::new(ReplitIdentityTokenManager),
                client: HttpClient::ReqwestClient(ReplitClient::new()),
            })
        } else {
            Ok(Self {
                server_url: server_url
                    .map_or_else(|| config.matador_url.clone(), ToString::to_string),
                auth: Box::new(L402TokenManager),
                client: HttpClient::L402Client(L402Client::new()), // Use L402Client
            })
        }
    }

    pub fn check_response(&self, response: &Response) -> Result<(), ApiError> {
        let status = response.status();

        if status == StatusCode::BAD_REQUEST {
            return Err(ApiError::InvalidRequest("Invalid request".to_string()));
        }

        if status != StatusCode::OK {
            return Err(ApiError::InvalidRequest(format!(
                "Invalid status code: {}",
                status
            )));
        }

        Ok(())
    }

    pub fn get_auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("{} {}", self.auth.get_token_type(), self.auth.get_token())
                .parse()
                .unwrap(),
        );
        headers
    }

    pub async fn check_streaming_response(&self, response: &Response) -> Result<(), ApiError> {
        let status = response.status();
        if status == StatusCode::OK {
            return Ok(());
        }
        self.check_response(&response)
    }
}
