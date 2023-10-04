use std::io::Error;

use bytes::Bytes;
use futures_util::Stream;
use lightning_invoice::{Bolt11Invoice, SignedRawBolt11Invoice};
use reqwest::{Client, Method, Request, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};

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

pub struct L402Client {
    pub client: Client,
    pub bolt11_endpoint: String,
    pub api_key: String,
    pub l402_token: Option<String>,
}

impl L402Client {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let l402_token = std::env::var("L402_TOKEN").ok();
        println!("Found l402_token: {:?}", l402_token);
        Self {
            client: Client::new(),
            bolt11_endpoint: std::env::var("LIGHTNING_API_ENDPOINT").unwrap(),
            api_key: std::env::var("LIGHTNING_API_KEY").unwrap(),
            l402_token: l402_token,
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
        let mut request = request;
        if let Some(token) = &self.l402_token {
            request
                .headers_mut()
                .insert("AUTHORIZATION", format!("L402 {}", token).parse().unwrap());
        } else {
            let request_copy = request.try_clone().unwrap();
            let response = self.client.execute(request).await?;
            if response.status() == StatusCode::PAYMENT_REQUIRED {
                println!("L402 Payment Required");
                println!(
                    "www-authenticate header: {:?}",
                    response.headers().get("www-authenticate")
                );
            }

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

            request = add_l402_header(request_copy, l402);
        }

        let response = self.client.execute(request).await.unwrap();

        Ok(response)
    }

    pub async fn execute_stream(
        &self,
        request: Request,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, reqwest::Error> {
        let response = self.execute(request).await?;
        Ok(response.bytes_stream())
    }

    pub async fn pay_invoice(&self, invoice: Bolt11Invoice) -> Result<String, Error> {
        println!("Paying invoice...");
        let alby_request = AlbyBolt11Request {
            invoice: invoice.to_string(),
            amount: None,
        };
        let alby_res: Response = self
            .post(&self.bolt11_endpoint)
            .bearer_auth(&self.api_key)
            .json(&alby_request)
            .send()
            .await
            .unwrap();

        let alby_res: AlbyBolt11Response =
            serde_json::from_str(&alby_res.text().await.unwrap()).unwrap();

        println!("Payment Preimage: {}", alby_res.payment_preimage);

        Ok(alby_res.payment_preimage)
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

    println!("Token: {}", token);
    println!("Invoice: {}", invoice);

    let invoice =
        Bolt11Invoice::from_signed(invoice.parse::<SignedRawBolt11Invoice>().unwrap()).unwrap();
    Ok(L402 {
        token: token.to_string(),
        invoice: Some(invoice),
        preimage: None,
    })
}

#[derive(Serialize)]
struct AlbyBolt11Request {
    invoice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<u64>,
}

#[derive(Deserialize)]
struct AlbyBolt11Response {
    payment_preimage: String,
}
