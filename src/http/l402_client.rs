use std::io::Error;

use bytes::Bytes;
use lightning_invoice::{Bolt11Invoice, SignedRawBolt11Invoice};
use log::info;
use reqwest::header::HeaderValue;
use reqwest::{Client, Method, Request, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

use super::{HttpClient, PinBoxStream};

#[derive(Debug, Clone)]
pub struct L402 {
    token: String,
    invoice: Option<Bolt11Invoice>,
    preimage: Option<String>,
}

impl L402 {
    pub fn new(token: String, invoice: Bolt11Invoice) -> Self {
        Self {
            token,
            invoice: Some(invoice),
            preimage: None,
        }
    }

    pub fn set_preimage(&mut self, preimage: String) {
        self.preimage = Some(preimage);
    }
}

#[derive(Debug, Clone)]
pub struct L402Client {
    pub client: Client,
    pub bolt11_endpoint: String,
    pub api_key: String,
    pub l402_token: Option<String>,
}

impl L402Client {
    pub fn new() -> Self {
        let (bolt11_endpoint, api_key, l402_token) = load_env_vars();
        // let client = ClientBuilder::new()
        //     .connection_verbose(true)
        //     .build()
        //     .unwrap();
        let client = Client::new();
        Self {
            client: client,
            bolt11_endpoint,
            api_key,
            l402_token,
        }
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client.request(method, url)
    }

    pub fn get_auth_header(&self) -> HeaderValue {
        format!("L402 {}", self.l402_token.clone().unwrap())
            .parse()
            .unwrap()
    }

    async fn handle_l402(
        &self,
        req_clone: Request,
        response: Response,
    ) -> Result<Response, reqwest::Error> {
        info!("L402 Payment Required");
        info!(
            "www-authenticate header: {:?}",
            response.headers().get("www-authenticate")
        );
        let mut l402 = parse_l402_header(
            response
                .headers()
                .get("www-authenticate")
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let preimage = self
            .pay_invoice(l402.clone().invoice.unwrap())
            .await
            .unwrap();
        l402.set_preimage(preimage);
        let request = add_l402_header(req_clone, l402);
        info!("Retrying with L402 Header");
        self.client.execute(request).await
    }

    pub async fn pay_invoice(&self, invoice: Bolt11Invoice) -> Result<String, Error> {
        info!("Paying invoice: {}", invoice.to_string());
        let request = self
            .client
            .post(self.bolt11_endpoint.as_str())
            .header("Authorization", self.api_key.clone())
            .json(&AlbyBolt11Request {
                invoice: invoice.to_string(),
                amount: None,
            })
            .build()
            .unwrap();

        let response = self.client.execute(request).await.unwrap();

        info!("Response: {:?}", response);

        let response: AlbyBolt11Response = response.json().await.unwrap();

        info!("Payment Preimage: {}", response.payment_preimage);

        Ok(response.payment_preimage)
    }
}

#[async_trait::async_trait(?Send)]
impl HttpClient for L402Client {
    fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    // Execute with L402 Handling
    async fn execute(&self, mut request: Request) -> Result<Response, reqwest::Error> {
        let req_clone = request.try_clone().unwrap();
        if self.l402_token.is_some() {
            request
                .headers_mut()
                .insert("AUTHORIZATION", self.get_auth_header());
        }
        let mut response = self.client.execute(request).await?;
        if response.status() == StatusCode::PAYMENT_REQUIRED {
            response = self.handle_l402(req_clone, response).await?;
        }

        Ok(response)
    }

    async fn execute_stream(&self, mut request: Request) -> PinBoxStream<Bytes> {
        if self.l402_token.is_some() {
            request
                .headers_mut()
                .insert("AUTHORIZATION", self.get_auth_header());
        };

        Box::pin(self.execute(request).await.unwrap().bytes_stream())
    }
}

pub fn add_l402_header(mut request: Request, l402: L402) -> Request {
    request.headers_mut().insert(
        "AUTHORIZATION",
        format!("L402 {}:{}", l402.token, l402.preimage.unwrap())
            .parse()
            .unwrap(),
    );

    request
}

pub fn parse_l402_header(header: &str) -> Result<L402, Error> {
    let mut parts = header.split(' ');
    // ignore L402
    parts.next();
    let token = parts
        .next()
        .unwrap_or_default()
        .replace("token=\"", "")
        .replace("\"", "")
        .replace(",", "");
    let invoice = parts
        .next()
        .unwrap_or_default()
        .replace("invoice=\"", "")
        .replace("\"", "");
    if token.is_empty() || invoice.is_empty() {
        return Err(Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid L402 Header",
        ));
    }

    info!("Token: {}", token);

    let invoice =
        Bolt11Invoice::from_signed(invoice.parse::<SignedRawBolt11Invoice>().unwrap()).unwrap();
    Ok(L402 {
        token: token.to_string(),
        invoice: Some(invoice),
        preimage: None,
    })
}

fn load_env_vars() -> (String, String, Option<String>) {
    dotenv::dotenv().ok();
    let bolt11_endpoint = std::env::var("LIGHTNING_API_ENDPOINT").unwrap();
    let api_key = std::env::var("LIGHTNING_API_KEY").unwrap();
    let l402_token = std::env::var("L402_TOKEN").ok();
    (bolt11_endpoint, api_key, l402_token)
}

#[derive(Serialize)]
struct AlbyBolt11Request {
    invoice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct AlbyBolt11Response {
    payment_preimage: String,
}
