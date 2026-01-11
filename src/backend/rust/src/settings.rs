use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default)]
    pub server: ServerSettings,
    #[serde(default)]
    pub database: DatabaseSettings,
    #[serde(default)]
    pub auth: AuthSettings,
    #[serde(default)]
    pub telemetry: TelemetrySettings,
}

fn default_environment() -> String {
    "production".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub secure_cookies: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default)]
    pub session_secret_key: String,
    #[serde(default = "default_rate_limit_period")]
    pub rate_limit_period_ms: u64,
    #[serde(default = "default_rate_limit_burst")]
    pub rate_limit_burst_size: u32,
}

fn default_rate_limit_period() -> u64 {
    50
}

fn default_rate_limit_burst() -> u32 {
    50
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub addr: Option<String>,
    #[serde(default = "default_db_name")]
    pub name: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AuthSettings {
    #[serde(default)]
    pub google_client_id: String,
    #[serde(default)]
    pub google_client_secret: String,
    pub google_redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetrySettings {
    #[serde(default = "default_otlp")]
    pub otlp_endpoint: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            port: default_port(),
            secure_cookies: false,
            allowed_origins: Vec::new(),
            session_secret_key: String::new(),
            rate_limit_period_ms: default_rate_limit_period(),
            rate_limit_burst_size: default_rate_limit_burst(),
        }
    }
}

fn default_port() -> u16 {
    8080
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            addr: None,
            name: default_db_name(),
        }
    }
}

fn default_db_name() -> String {
    "MinesweeperGame".to_string()
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            otlp_endpoint: default_otlp(),
        }
    }
}

fn default_otlp() -> String {
    "http://signoz-otel-collector:4317".to_string()
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        // Manual overrides for legacy flat environment variables
        if let Ok(env) = env::var("APP_ENVIRONMENT") {
            builder = builder.set_override("environment", env)?;
        }
        if let Ok(port) = env::var("PORT") {
            if let Ok(port) = port.parse::<u16>() {
                builder = builder.set_override("server.port", port)?;
            }
        }
        if let Ok(secure) = env::var("SECURE_COOKIES") {
            builder =
                builder.set_override("server.secure_cookies", secure.to_lowercase() == "true")?;
        }
        if let Ok(origins) = env::var("ALLOWED_ORIGINS") {
            let origins: Vec<String> = origins.split(',').map(|s| s.to_string()).collect();
            builder = builder.set_override("server.allowed_origins", origins)?;
        }
        if let Ok(period) = env::var("RATE_LIMIT_PERIOD_MS") {
            if let Ok(period) = period.parse::<u64>() {
                builder = builder.set_override("server.rate_limit_period_ms", period)?;
            }
        }
        if let Ok(burst) = env::var("RATE_LIMIT_BURST_SIZE") {
            if let Ok(burst) = burst.parse::<u32>() {
                builder = builder.set_override("server.rate_limit_burst_size", burst)?;
            }
        }
        if let Ok(key) = env::var("SESSION_SECRET_KEY") {
            builder = builder.set_override("server.session_secret_key", key)?;
        }
        if let Ok(addr) = env::var("DB_ADDR") {
            builder = builder.set_override("database.addr", addr)?;
        }
        if let Ok(id) = env::var("GOOGLE_CLIENT_ID") {
            builder = builder.set_override("auth.google_client_id", id)?;
        }
        if let Ok(secret) = env::var("GOOGLE_CLIENT_SECRET") {
            builder = builder.set_override("auth.google_client_secret", secret)?;
        }
        if let Ok(uri) = env::var("GOOGLE_REDIRECT_URI") {
            builder = builder.set_override("auth.google_redirect_uri", uri)?;
        }
        if let Ok(endpoint) = env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            builder = builder.set_override("telemetry.otlp_endpoint", endpoint)?;
        }

        let settings: Self = builder
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()?;

        settings.validate().map_err(ConfigError::Message)?;

        Ok(settings)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.session_secret_key.len() < 64 {
            return Err("SESSION_SECRET_KEY must be at least 64 characters long".into());
        }
        if self.auth.google_client_id.is_empty() {
            return Err("GOOGLE_CLIENT_ID must be set".into());
        }
        if self.auth.google_client_secret.is_empty() {
            return Err("GOOGLE_CLIENT_SECRET must be set".into());
        }
        Ok(())
    }
}
