use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use url::Url;

use super::error::LightningError;
use super::model::CreateInvoiceParams;

#[derive(Clone)]
pub struct StrikeClient {
    api_key: String,
    strike_url: Url,
    reqwest_client: reqwest::Client,
}

impl StrikeClient {
    pub fn new(api_key: &str) -> Result<StrikeClient, LightningError> {
        let strike_url = Url::parse("https://api.strike.me")?;

        let reqwest_client = reqwest::Client::builder().build()?;

        Ok(StrikeClient {
            api_key: api_key.to_owned(),
            strike_url,
            reqwest_client,
        })
    }
}

impl StrikeClient {
    pub async fn make_get(&self, endpoint: &str) -> Result<String, LightningError> {
        let url = self.strike_url.join(endpoint)?;
        let response = self
            .reqwest_client
            .get(url)
            .bearer_auth(self.api_key.clone())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(LightningError::NotFound);
        }

        Ok(response.text().await?)
    }

    pub async fn make_post(&self, endpoint: &str, body: &str) -> Result<String, LightningError> {
        let url = self.strike_url.join(endpoint)?;
        let response = self
            .reqwest_client
            .post(url)
            .bearer_auth(self.api_key.clone())
            .header(
                CONTENT_TYPE,
                HeaderValue::from_str("application/json").expect("Invalid header value"),
            )
            .body(body.to_string())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(LightningError::NotFound);
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(LightningError::Unauthorized);
        }

        Ok(response.text().await?)
    }

    pub async fn make_patch(&self, endpoint: &str, body: &str) -> Result<String, LightningError> {
        let url = self.strike_url.join(endpoint)?;
        let response = self
            .reqwest_client
            .patch(url)
            .bearer_auth(self.api_key.clone())
            .header(
                CONTENT_TYPE,
                HeaderValue::from_str("application/json").expect("Invalid header value"),
            )
            .body(body.to_string())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(LightningError::NotFound);
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(LightningError::Unauthorized);
        }

        Ok(response.text().await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteRequest {
    #[serde(rename = "descriptionHash")]
    pub description_hash: String,
}

// strike has a 2 step process for getting a lightning invoice
// 1. create an "invoice" which on their platform means a currency agnostic
//    payment request
// 2. generate a "quote" for the invoice which is a specific quoted conversion
//    rate and a lightning invoice
impl StrikeClient {
    // this is not a lightning invoice, it's the strike internal representation of
    // an invoice
    pub async fn create_strike_invoice(
        &self,
        params: &CreateInvoiceParams,
    ) -> Result<String, LightningError> {
        let btc = (params.amount as f64) / 100_000_000.0;
        let params = serde_json::json!({
            "amount": {
                "amount": btc,
                "currency": "BTC"
            },
            "description": params.memo,
        });
        let body = self
            .make_post("v1/invoices", &serde_json::to_string(&params)?)
            .await?;

        let response: serde_json::Value = serde_json::from_str(&body)?;
        let invoice_id = response["invoiceId"]
            .as_str()
            .expect("invoiceId is empty")
            .to_owned();

        Ok(invoice_id)
    }

    // this is how you get the actual lightning invoice
    pub async fn create_strike_quote(&self, invoice_id: &str) -> Result<String, LightningError> {
        let endpoint = format!("v1/invoices/{}/quote", invoice_id);
        let description_hash = format!(
            "{:0>64}",
            hex::encode(hex::decode(invoice_id.replace('-', "").as_bytes()).unwrap())
        );
        let params = QuoteRequest { description_hash };
        let body = self
            .make_post(&endpoint, &serde_json::to_string(&params)?)
            .await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;
        let payment_request = response["lnInvoice"]
            .as_str()
            .expect("lnInvoice is empty")
            .to_owned();

        Ok(payment_request)
    }

    pub async fn create_ln_payment_quote(&self, bolt11: &str) -> Result<String, LightningError> {
        let params = serde_json::json!({
            "lnInvoice": bolt11,
            "sourceCurrency": "BTC",
        });
        let body = self
            .make_post(
                "v1/payment-quotes/lightning",
                &serde_json::to_string(&params)?,
            )
            .await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;
        let payment_quote_id = response["paymentQuoteId"]
            .as_str()
            .expect("paymentQuoteId is empty")
            .to_owned();

        Ok(payment_quote_id)
    }

    pub async fn execute_ln_payment_quote(&self, quote_id: &str) -> Result<bool, LightningError> {
        let endpoint = format!("v1/payment-quotes/{}/execute", quote_id);
        let body = self
            .make_patch(&endpoint, &serde_json::to_string(&serde_json::json!({}))?)
            .await?;
        let response: serde_json::Value = serde_json::from_str(&body)?;

        Ok(response["state"].as_str().unwrap_or("") == "COMPLETED")
    }

    pub async fn is_invoice_paid(&self, invoice_id: &str) -> Result<bool, LightningError> {
        let body = self.make_get(&format!("v1/invoices/{invoice_id}")).await?;
        let response = serde_json::from_str::<serde_json::Value>(&body)?;

        Ok(response["state"].as_str().unwrap_or("") == "PAID")
    }
}
