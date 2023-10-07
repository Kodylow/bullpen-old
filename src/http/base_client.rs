use std::pin::Pin;

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use reqwest::{Request, Response};

use super::{L402Client, ReplitClient};

pub type PinBoxStream<T> =
    Pin<Box<dyn futures_util::stream::Stream<Item = Result<T, reqwest::Error>>>>;

#[async_trait::async_trait(?Send)]
pub trait HttpClient {
    fn post(&self, url: &str) -> reqwest::RequestBuilder;
    fn get(&self, url: &str) -> reqwest::RequestBuilder;
    async fn execute(&self, request: Request) -> Result<Response, reqwest::Error>;
    async fn execute_stream(&self, request: Request) -> PinBoxStream<Bytes>;
}
