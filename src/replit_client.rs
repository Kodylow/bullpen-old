use std::io::Error;

use bytes::Bytes;
use futures_util::Stream;
use lightning_invoice::{Bolt11Invoice, SignedRawBolt11Invoice};
use reqwest::{Client, Method, Request, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

pub struct ReplitClient {
    pub client: Client,
    pub api_key: String,
}

impl ReplitClient {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self {
            client: Client::new(),
            api_key: std::env::var("LIGHTNING_API_KEY").unwrap(),
        }
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client.request(method, url)
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    pub fn get_auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    // Execute with L402 Handling
    pub async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        let mut request = request;
        request
            .headers_mut()
            .insert("AUTHORIZATION", self.get_auth_header().parse().unwrap());

        let response = self.client.execute(request).await.unwrap();

        Ok(response)
    }

    pub async fn execute_stream(
        &self,
        mut request: Request,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, reqwest::Error> {
        request
            .headers_mut()
            .insert("AUTHORIZATION", self.get_auth_header().parse().unwrap());

        let response = self.client.execute(request).await?;

        // Print the Transfer-Encoding header
        println!(
            "Transfer-Encoding: {:?}",
            response.headers().get("transfer-encoding")
        );

        let stream = response.bytes_stream();

        Ok(stream)
    }
}
