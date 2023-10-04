use reqwest::{Client, Method, Request, RequestBuilder, Response, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

pub struct L402Client {
    pub client: Client,
}

impl L402Client {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
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

    // Execute with L402 Handling
    pub async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        let response = self.client.execute(request).await?;
        if response.status() == StatusCode::PAYMENT_REQUIRED {
            println!("L402 Payment Required");
            println!(
                "www-authenticate header: {:?}",
                response.headers().get("www-authenticate")
            );
        }

        Ok(response)
    }
}
