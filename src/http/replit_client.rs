use bytes::Bytes;

use reqwest::{Client, Method, Request, RequestBuilder, Response};

use super::{HttpClient, PinBoxStream};
use crate::token_manager::generate_replit_key;

pub struct ReplitClient {
    pub client: Client,
    pub api_key: String,
}

impl ReplitClient {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self {
            client: Client::new(),
            api_key: generate_replit_key(),
        }
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client.request(method, url)
    }

    pub fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }
}

#[async_trait::async_trait(?Send)]
impl HttpClient for ReplitClient {
    fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        let mut request = request;
        request
            .headers_mut()
            .insert("AUTHORIZATION", self.get_auth_header().parse().unwrap());

        let response = self.client.execute(request).await.unwrap();

        Ok(response)
    }

    async fn execute_stream(&self, request: Request) -> PinBoxStream<Bytes> {
        let mut request = request;
        request
            .headers_mut()
            .insert("AUTHORIZATION", self.get_auth_header().parse().unwrap());

        Box::pin(self.client.execute(request).await.unwrap().bytes_stream())
    }
}
