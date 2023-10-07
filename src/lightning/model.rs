use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceParams {
    pub amount: u64,
    pub unit: String,
    pub memo: Option<String>,
    pub expiry: Option<u32>,
    pub webhook: Option<String>,
    pub internal: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceResult {
    pub payment_hash: Vec<u8>,
    pub payment_request: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayInvoiceResult {
    pub payment_hash: String,
}
