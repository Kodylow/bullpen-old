use std::fmt::{self, Formatter};

use async_trait::async_trait;
use cln_rpc::primitives::{Amount, AmountOrAny};
use cln_rpc::ClnRpc;
use lightning_invoice::{Bolt11Invoice as LNInvoice, SignedRawBolt11Invoice};
use secp256k1::bitcoin_hashes::Hash;
use secp256k1::rand;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use tonic_lnd::Client;
use url::Url;
mod alby;
pub mod error;
mod lnbits;
mod model;
mod strike;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use self::alby::AlbyClient;
use self::error::LightningError;
use self::lnbits::LNBitsClient;
use self::model::{CreateInvoiceParams, CreateInvoiceResult, PayInvoiceResult};
use self::strike::StrikeClient;

#[derive(Debug, Clone)]
pub enum LightningType {
    Lnbits(LnbitsLightningSettings),
    Alby(AlbyLightningSettings),
    Strike(StrikeLightningSettings),
    Lnd(LndLightningSettings),
    Cln(ClnLightningSettings),
}

impl fmt::Display for LightningType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LightningType::Lnbits(settings) => write!(f, "Lnbits: {}", settings),
            LightningType::Alby(settings) => write!(f, "Alby: {}", settings),
            LightningType::Strike(settings) => write!(f, "Strike: {}", settings),
            LightningType::Lnd(settings) => write!(f, "Lnd: {}", settings),
            LightningType::Cln(settings) => write!(f, "Cln: {}", settings),
        }
    }
}

#[async_trait]
pub trait Lightning: Send + Sync {
    async fn is_invoice_paid(&self, invoice: String) -> Result<bool, anyhow::Error>;
    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, anyhow::Error>;
    async fn pay_invoice(&self, payment_request: String)
        -> Result<PayInvoiceResult, anyhow::Error>;

    async fn decode_invoice(&self, payment_request: String) -> Result<LNInvoice, anyhow::Error> {
        LNInvoice::from_str(&payment_request).map_err(|err| {
            anyhow::anyhow!(
                "Failed to decode invoice: {}, error: {}",
                payment_request,
                err
            )
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct LnbitsLightningSettings {
    pub admin_key: Option<String>,
    pub url: Option<String>, // FIXME use Url type instead
}

impl LnbitsLightningSettings {
    pub fn new(admin_key: &str, url: &str) -> Self {
        Self {
            admin_key: Some(admin_key.to_owned()),
            url: Some(url.to_owned()),
        }
    }
}

impl fmt::Display for LnbitsLightningSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "admin_key: {}, url: {}",
            self.admin_key.as_ref().unwrap(),
            self.url.as_ref().unwrap()
        )
    }
}

#[derive(Clone)]
pub struct LnbitsLightning {
    pub client: LNBitsClient,
}

impl LnbitsLightning {
    pub fn new(admin_key: String, url: String) -> Self {
        Self {
            client: LNBitsClient::new(&admin_key, &url, None)
                .expect("Can not create Lnbits client"),
        }
    }
}

#[async_trait]
impl Lightning for LnbitsLightning {
    async fn is_invoice_paid(&self, invoice: String) -> Result<bool, anyhow::Error> {
        let decoded_invoice = self.decode_invoice(invoice).await?;
        Ok(self
            .client
            .is_invoice_paid(&decoded_invoice.payment_hash().to_string())
            .await?)
    }

    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, anyhow::Error> {
        Ok(self
            .client
            .create_invoice(&CreateInvoiceParams {
                amount,
                unit: "sat".to_string(),
                memo: None,
                expiry: Some(10000),
                webhook: None,
                internal: None,
            })
            .await?)
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        self.client
            .pay_invoice(&payment_request)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to pay invoice: {}", err))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AlbyLightningSettings {
    pub api_key: Option<String>,
}

impl fmt::Display for AlbyLightningSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "api_key: {}", self.api_key.as_ref().unwrap(),)
    }
}

impl AlbyLightningSettings {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_owned()),
        }
    }
}

#[derive(Clone)]
pub struct AlbyLightning {
    pub client: AlbyClient,
}

impl AlbyLightning {
    pub fn new(api_key: String) -> Self {
        Self {
            client: AlbyClient::new(&api_key).expect("Can not create Alby client"),
        }
    }
}
#[async_trait]
impl Lightning for AlbyLightning {
    async fn is_invoice_paid(&self, invoice: String) -> Result<bool, anyhow::Error> {
        let decoded_invoice = self.decode_invoice(invoice).await?;
        Ok(self
            .client
            .is_invoice_paid(&decoded_invoice.payment_hash().to_string())
            .await?)
    }

    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, anyhow::Error> {
        Ok(self
            .client
            .create_invoice(&CreateInvoiceParams {
                amount,
                unit: "sat".to_string(),
                memo: None,
                expiry: Some(10000),
                webhook: None,
                internal: None,
            })
            .await?)
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        self.client
            .pay_invoice(&payment_request)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to pay invoice: {}", err))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct StrikeLightningSettings {
    pub api_key: Option<String>,
}

impl fmt::Display for StrikeLightningSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "api_key: {}", self.api_key.as_ref().unwrap(),)
    }
}

impl StrikeLightningSettings {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_owned()),
        }
    }
}

#[derive(Clone)]
pub struct StrikeLightning {
    pub client: StrikeClient,
}

impl StrikeLightning {
    pub fn new(api_key: String) -> Self {
        Self {
            client: StrikeClient::new(&api_key).expect("Can not create Strike client"),
        }
    }
}

#[async_trait]
impl Lightning for StrikeLightning {
    async fn is_invoice_paid(&self, invoice: String) -> Result<bool, anyhow::Error> {
        let decoded_invoice = self.decode_invoice(invoice).await?;
        let description_hash = decoded_invoice
            .into_signed_raw()
            .description_hash()
            .unwrap()
            .0;

        // invoiceId is the last 16 bytes of the description hash
        let invoice_id = format_as_uuid_string(&description_hash[16..]);

        Ok(self.client.is_invoice_paid(&invoice_id).await?)
    }

    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, anyhow::Error> {
        let strike_invoice_id = self
            .client
            .create_strike_invoice(&CreateInvoiceParams {
                amount,
                unit: "sat".to_string(),
                memo: None,
                expiry: Some(10000),
                webhook: None,
                internal: None,
            })
            .await?;

        let payment_request = self.client.create_strike_quote(&strike_invoice_id).await?;
        // strike doesn't return the payment_hash so we have to read the invoice into a
        // Bolt11 and extract it
        let invoice =
            LNInvoice::from_signed(payment_request.parse::<SignedRawBolt11Invoice>().unwrap())
                .unwrap();
        let payment_hash = invoice.payment_hash().to_vec();

        Ok(CreateInvoiceResult {
            payment_hash,
            payment_request,
        })
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        // strike doesn't return the payment_hash so we have to read the invoice into a
        // Bolt11 and extract it
        let invoice = self.decode_invoice(payment_request.clone()).await?;
        let payment_hash = invoice.payment_hash().to_vec();

        let payment_quote_id = self
            .client
            .create_ln_payment_quote(&invoice.into_signed_raw().to_string())
            .await?;

        let payment_result = self
            .client
            .execute_ln_payment_quote(&payment_quote_id)
            .await?;

        if !payment_result {
            return Err(anyhow::anyhow!(
                "Failed to pay invoice: {}",
                payment_request
            ));
        }

        Ok(PayInvoiceResult {
            payment_hash: hex::encode(payment_hash),
        })
    }
}

fn format_as_uuid_string(bytes: &[u8]) -> String {
    let byte_str = hex::encode(bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &byte_str[..8],
        &byte_str[8..12],
        &byte_str[12..16],
        &byte_str[16..20],
        &byte_str[20..]
    )
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let url_str: Option<String> = Option::deserialize(deserializer)?;
    match url_str {
        Some(s) => Url::parse(&s).map_err(serde::de::Error::custom).map(Some),
        None => Ok(None),
    }
}

fn serialize_url<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match url {
        Some(url) => serializer.serialize_str(url.as_str()),
        None => serializer.serialize_none(),
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct LndLightningSettings {
    #[serde(serialize_with = "serialize_url", deserialize_with = "deserialize_url")]
    pub grpc_host: Option<Url>,
    pub tls_cert_path: Option<PathBuf>,
    pub macaroon_path: Option<PathBuf>,
}
impl fmt::Display for LndLightningSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "grpc_host: {}, tls_cert_path: {}, macaroon_path: {}",
            self.grpc_host.as_ref().unwrap(),
            self.tls_cert_path
                .as_ref()
                .unwrap() // FIXME unwrap
                .to_str()
                .unwrap_or_default(),
            self.macaroon_path
                .as_ref()
                .unwrap()
                .to_str()
                .unwrap_or_default()
        )
    }
}

pub struct LndLightning(Arc<Mutex<Client>>);

impl LndLightning {
    pub async fn new(
        address: Url,
        cert_file: &PathBuf,
        macaroon_file: &PathBuf,
    ) -> Result<Self, anyhow::Error> {
        let client = tonic_lnd::connect(address.to_string(), cert_file, &macaroon_file).await;

        Ok(Self(Arc::new(Mutex::new(client.unwrap()))))
    }

    pub async fn client_lock(
        &self,
    ) -> anyhow::Result<MappedMutexGuard<'_, tonic_lnd::LightningClient>> {
        let guard = self.0.lock().await;
        Ok(MutexGuard::map(guard, |client| client.lightning()))
    }
}

#[async_trait]
impl Lightning for LndLightning {
    async fn is_invoice_paid(&self, payment_request: String) -> Result<bool, anyhow::Error> {
        let invoice = self.decode_invoice(payment_request).await?;
        let payment_hash = invoice.payment_hash();
        let invoice_request = tonic_lnd::lnrpc::PaymentHash {
            r_hash: payment_hash.to_vec(),
            ..Default::default()
        };

        let invoice = self
            .client_lock()
            .await
            .expect("failed to lock client")
            .lookup_invoice(tonic_lnd::tonic::Request::new(invoice_request))
            .await
            .expect("failed to lookup invoice")
            .into_inner();

        Ok(invoice.state == tonic_lnd::lnrpc::invoice::InvoiceState::Settled as i32)
    }

    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, anyhow::Error> {
        let invoice_request = tonic_lnd::lnrpc::Invoice {
            value: amount as i64,
            ..Default::default()
        };

        let invoice = self
            .client_lock()
            .await
            .expect("failed to lock client")
            .add_invoice(tonic_lnd::tonic::Request::new(invoice_request))
            .await
            .expect("failed to create invoice")
            .into_inner();

        Ok(CreateInvoiceResult {
            payment_hash: invoice.r_hash,
            payment_request: invoice.payment_request,
        })
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        let pay_req = tonic_lnd::lnrpc::SendRequest {
            payment_request,
            ..Default::default()
        };
        let payment = self
            .client_lock()
            .await
            .expect("failed to lock client") //FIXME map error
            .send_payment_sync(tonic_lnd::tonic::Request::new(pay_req))
            .await
            .expect("failed to pay invoice")
            .into_inner();

        Ok(PayInvoiceResult {
            payment_hash: hex::encode(payment.payment_hash),
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ClnLightningSettings {
    pub rpc_path: Option<PathBuf>,
}
impl fmt::Display for ClnLightningSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rpc_path: {}",
            self.rpc_path.as_ref().unwrap().to_str().unwrap_or_default()
        )
    }
}

pub struct ClnLightning(Arc<Mutex<ClnRpc>>);

impl ClnLightning {
    pub async fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let client = ClnRpc::new(path).await;

        Ok(Self(Arc::new(Mutex::new(
            client.expect("failed to create client"),
        ))))
    }

    pub async fn client_lock(&self) -> anyhow::Result<MappedMutexGuard<'_, ClnRpc>> {
        let guard = self.0.lock().await;
        Ok(MutexGuard::map(guard, |client| client))
    }
}

#[async_trait]
impl Lightning for ClnLightning {
    async fn is_invoice_paid(&self, payment_request: String) -> Result<bool, anyhow::Error> {
        let invoices = self
            .client_lock()
            .await
            .expect("failed to lock client")
            .call_typed(cln_rpc::model::requests::ListinvoicesRequest {
                invstring: Some(payment_request),
                label: None,
                payment_hash: None,
                offer_id: None,
                index: None,
                start: None,
                limit: None,
            })
            .await
            .expect("failed to lookup invoice");
        let invoice = invoices
            .invoices
            .first()
            .expect("no matching invoice found");

        Ok(invoice.status == cln_rpc::model::responses::ListinvoicesInvoicesStatus::PAID)
    }

    async fn create_invoice(&self, amount: u64) -> Result<CreateInvoiceResult, anyhow::Error> {
        let invoice = self
            .client_lock()
            .await
            .expect("failed to lock client")
            .call_typed(cln_rpc::model::requests::InvoiceRequest {
                amount_msat: AmountOrAny::Amount(Amount::from_sat(amount)),
                description: format!("{:x}", rand::random::<u128>()),
                label: format!("{:x}", rand::random::<u128>()),
                expiry: None,
                fallbacks: None,
                preimage: None,
                cltv: None,
                deschashonly: None,
            })
            .await
            .expect("failed to create invoice");

        Ok(CreateInvoiceResult {
            payment_hash: invoice.payment_hash.to_byte_array(),
            payment_request: invoice.bolt11,
        })
    }

    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        let payment = self
            .client_lock()
            .await
            .expect("failed to lock client") //FIXME map error
            .call_typed(cln_rpc::model::requests::PayRequest {
                bolt11: payment_request,
                amount_msat: None,
                label: None,
                riskfactor: None,
                maxfeepercent: None,
                retry_for: None,
                maxdelay: None,
                exemptfee: None,
                localinvreqid: None,
                exclude: None,
                maxfee: None,
                description: None,
            })
            .await
            .expect("failed to pay invoice");

        Ok(PayInvoiceResult {
            payment_hash: hex::encode(payment.payment_hash),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::lightning::{Lightning, LnbitsLightning};

    #[tokio::test]
    async fn test_decode_invoice() -> anyhow::Result<()> {
        let invoice = "lnbcrt55550n1pjga687pp5ac8ja6n5hn90huztxxp746w48vtj8ys5uvze6749dvcsd5j5sdvsdqqcqzzsxqyz5vqsp5kzzq0ycxspxjygsxkfkexkkejjr5ggeyl56mwa7s0ygk2q8z92ns9qyyssqt7myq7sryffasx8v47al053ut4vqts32e9hvedvs7eml5h9vdrtj3k5m72yex5jv355jpuzk2xjjn5468cz87nhp50jyr2al2a5zjvgq2xs5uq".to_string();

        let lightning =
            LnbitsLightning::new("admin_key".to_string(), "http://localhost:5000".to_string());

        let decoded_invoice = lightning.decode_invoice(invoice).await?;
        assert_eq!(
            decoded_invoice
                .amount_milli_satoshis()
                .expect("invalid amount"),
            5_555 * 1_000
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_decode_invoice_invalid() -> anyhow::Result<()> {
        let invoice = "lnbcrt55550n1pjga689pp5ac8ja6n5hn90huztyxp746w48vtj8ys5uvze6749dvcsd5j5sdvsdqqcqzzsxqyz5vqsp5kzzq0ycxspxjygsxkfkexkkejjr5ggeyl56mwa7s0ygk2q8z92ns9qyyssqt7myq7sryffasx8v47al053ut4vqts32e9hvedvs7eml5h9vdrtj3k5m72yex5jv355jpuzk2xjjn5468cz87nhp50jyr2al2a5zjvgq2xs5uw".to_string();

        let lightning =
            LnbitsLightning::new("admin_key".to_string(), "http://localhost:5000".to_string());

        let decoded_invoice = lightning.decode_invoice(invoice).await;
        assert!(decoded_invoice.is_err());
        Ok(())
    }
}
