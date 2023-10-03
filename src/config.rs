use once_cell::sync::Lazy;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub root_url: String,
    pub matador_url: String,
    pub audience: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            root_url: "https://production-modelfarm.replit.com".to_string(),
            matador_url: "https://matador-replit.kody.repl.co/replit".to_string(),
            audience: "modelfarm@replit.com".to_string(),
        }
    }

    pub fn initialize(
        &mut self,
        root_url: Option<&str>,
        matador_url: Option<&str>,
        server_audience: Option<&str>,
    ) {
        if let Some(url) = root_url {
            self.root_url = url.to_string();
        }
        if let Some(url) = matador_url {
            self.matador_url = url.to_string();
        }
        if let Some(aud) = server_audience {
            self.audience = aud.to_string();
        }
    }
}

static GLOBAL_CONFIG: Lazy<std::sync::Mutex<Config>> =
    Lazy::new(|| std::sync::Mutex::new(Config::new()));

pub fn get_config() -> std::sync::MutexGuard<'static, Config> {
    GLOBAL_CONFIG.lock().unwrap()
}

pub fn initialize(
    root_url: Option<&str>,
    matador_url: Option<&str>,
    server_audience: Option<&str>,
) {
    let mut config = GLOBAL_CONFIG.lock().unwrap();
    config.initialize(root_url, matador_url, server_audience);
}
