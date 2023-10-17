use std::fmt::{self, Formatter};

use async_trait::async_trait;

use cln_rpc::ClnRpc;


use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use tonic_lnd::Client;
use url::Url;
mod alby;
pub mod error;
mod lnbits;
mod model;
mod strike;
pub mod utils;

use std::path::PathBuf;

use std::sync::Arc;

use self::alby::AlbyClient;

use self::lnbits::LNBitsClient;
use self::model::{PayInvoiceResult};
use self::strike::StrikeClient;
use self::utils::decode_invoice;

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
    async fn pay_invoice(&self, payment_request: String)
        -> Result<PayInvoiceResult, anyhow::Error>;
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
    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        // strike doesn't return the payment_hash so we have to read the invoice into a
        // Bolt11 and extract it
        let invoice = decode_invoice(payment_request.clone());
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
        let client = ClnRpc::new(path)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to create client: {}", err))?;

        Ok(Self(Arc::new(Mutex::new(client))))
    }

    pub async fn client_lock(&self) -> anyhow::Result<MappedMutexGuard<'_, ClnRpc>> {
        let guard = self.0.lock().await;
        Ok(MutexGuard::map(guard, |client| client))
    }
}

#[async_trait]
impl Lightning for ClnLightning {
    async fn pay_invoice(
        &self,
        payment_request: String,
    ) -> Result<PayInvoiceResult, anyhow::Error> {
        let payment = self
            .client_lock()
            .await?
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
            .map_err(|err| anyhow::anyhow!("Failed to pay invoice: {}", err))?;

        Ok(PayInvoiceResult {
            payment_hash: hex::encode(payment.payment_hash),
        })
    }
}
