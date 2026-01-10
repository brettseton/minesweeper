use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub auth: AuthSettings,
    pub telemetry: TelemetrySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub port: u16,
    pub secure_cookies: bool,
    pub allowed_origins: Vec<String>,
    pub session_secret_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub addr: Option<String>,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthSettings {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetrySettings {
    pub otlp_endpoint: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            // Start with default values
            .set_default("server.port", 8080)?
            .set_default("server.secure_cookies", false)?
            .set_default("server.allowed_origins", Vec::<String>::new())?
            .set_default("server.session_secret_key", "a".repeat(64))?
            .set_default("database.name", "MinesweeperGame")?
            .set_default(
                "telemetry.otlp_endpoint",
                "http://signoz-otel-collector:4317",
            )?;

        // Manual overrides for legacy flat environment variables
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

        builder
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
