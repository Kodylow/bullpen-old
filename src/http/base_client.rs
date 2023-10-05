use bytes::Bytes;
use futures_util::Stream;

use reqwest::{Request, Response};

use super::{L402Client, ReplitClient};

pub enum HttpClient {
    // ReqwestClient(ReplitClient),
    L402Client(L402Client),
}

impl HttpClient {
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        match self {
            // HttpClient::ReqwestClient(client) => client.post(url),
            HttpClient::L402Client(client) => client.post(url),
        }
    }

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        match self {
            // HttpClient::ReqwestClient(client) => client.get(url),
            HttpClient::L402Client(client) => client.get(url),
        }
    }

    pub async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        match self {
            // HttpClient::ReqwestClient(client) => client.execute(request).await,
            HttpClient::L402Client(client) => client.execute(request).await,
        }
    }

    pub async fn execute_stream(
        &self,
        request: Request,
    ) -> impl Stream<Item = Result<Bytes, reqwest::Error>> {
        match self {
            // HttpClient::ReqwestClient(client) => client.execute_stream(request).await,
            HttpClient::L402Client(client) => client.execute_stream(request).await,
        }
    }
}
