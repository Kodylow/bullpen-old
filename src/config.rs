use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub enum LightningBackend {
    Alby,
}

impl Default for LightningBackend {
    fn default() -> Self {
        LightningBackend::Alby
    }
}

impl std::fmt::Display for LightningBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LightningBackend::Alby => write!(f, "alby"),
        }
    }
}

impl LightningBackend {
    fn endpoint(&self) -> String {
        match self {
            LightningBackend::Alby => "https://api.getalby.com/payments/bolt11".to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub root_url: String,
    pub matador_url: String,
    pub audience: String,
    pub lightning_backend: LightningBackend,
}

impl Config {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Config {
            root_url: "https://production-modelfarm.replit.com".to_string(),
            matador_url: "https://matador-replit.kody.repl.co/replit".to_string(),
            audience: "modelfarm@replit.com".to_string(),
            lightning_backend: LightningBackend::default(),
        }
    }
}

static GLOBAL_CONFIG: Lazy<std::sync::Mutex<Config>> =
    Lazy::new(|| std::sync::Mutex::new(Config::new()));

pub fn get_config() -> std::sync::MutexGuard<'static, Config> {
    GLOBAL_CONFIG.lock().unwrap()
}
