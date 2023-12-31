use std::env;

use anyhow::anyhow;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode};

use crate::http::{HttpClient, L402Client, ReplitClient};
use crate::token_manager::{L402TokenManager, ReplitIdentityTokenManager, TokenManager};

pub struct Model {
    pub server_url: String,
    pub auth: Box<dyn TokenManager>,
    pub client: Box<dyn HttpClient>,
}

pub trait ModelTrait {
    fn new(server_url: Option<&str>) -> Result<Self, anyhow::Error>
    where
        Self: Sized;

    fn check_response(&self, response: &Response) -> Result<(), anyhow::Error>;

    fn get_auth_headers(&self) -> HeaderMap;

    fn check_streaming_response(&self, response: &Response) -> Result<(), anyhow::Error>;
}

impl Model {
    pub fn new(server_url: Option<&str>) -> Result<Self, anyhow::Error> {
        let config = crate::config::get_config();
        if env::var("REPLIT_DEPLOYMENT").is_ok()
            || env::var("REPL_IDENTITY_KEY").is_ok()
            || env::var("REPL_IDENTITY").is_ok()
            || env::var("REPL_ID").is_ok()
        {
            Ok(Self {
                server_url: server_url.map_or_else(|| config.root_url.clone(), ToString::to_string),
                auth: Box::new(ReplitIdentityTokenManager),
                client: Box::new(ReplitClient::new()),
            })
        } else {
            Ok(Self {
                server_url: server_url
                    .map_or_else(|| config.matador_url.clone(), ToString::to_string),
                auth: Box::new(L402TokenManager),
                client: Box::new(L402Client::new()),
            })
        }
    }

    pub fn check_response(&self, response: &Response) -> Result<(), anyhow::Error> {
        let status = response.status();

        if status == StatusCode::BAD_REQUEST {
            return Err(anyhow!("Invalid request"));
        }

        if status != StatusCode::OK {
            return Err(anyhow!("Invalid response"));
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

    pub async fn check_streaming_response(&self, response: &Response) -> Result<(), anyhow::Error> {
        let status = response.status();
        if status == StatusCode::OK {
            return Ok(());
        }
        self.check_response(&response)
    }
}
