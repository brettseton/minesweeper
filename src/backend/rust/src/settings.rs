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
    pub redis: RedisSettings,
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
    /// Public, external origin (scheme + host + optional port), e.g. `https://minesweeper.example.com`.
    /// When set, this is used for CSRF origin checks and as the base for OAuth callback URLs.
    pub public_origin: Option<String>,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default = "default_cors_supports_credentials")]
    pub cors_supports_credentials: bool,
    /// When true, configure cookies for cross-origin session use (SameSite=None; Secure).
    /// This requires HTTPS and credentialed CORS with an explicit origin allowlist.
    #[serde(default)]
    pub cross_origin_cookies: bool,
    /// List of trusted proxy IPs or CIDRs (e.g. `10.0.0.1`, `10.0.0.0/8`, `fd00::/8`).
    /// Forwarded headers are only honored when the peer address matches one of these entries.
    #[serde(default)]
    pub trusted_proxies: Vec<String>,
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

fn default_cors_supports_credentials() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub addr: Option<String>,
    #[serde(default = "default_db_name")]
    pub name: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RedisSettings {
    pub addr: Option<String>,
    #[serde(default = "default_redis_ttl_seconds")]
    pub ttl_seconds: u64,
    #[serde(default = "default_snapshot_interval_seconds")]
    pub snapshot_interval_seconds: u64,
    #[serde(default = "default_redis_required_for_writes")]
    pub required_for_writes: bool,
    #[serde(default = "default_redis_required")]
    pub required: bool,
}

fn default_redis_ttl_seconds() -> u64 {
    24 * 60 * 60
}

fn default_snapshot_interval_seconds() -> u64 {
    60
}

fn default_redis_required_for_writes() -> bool {
    true
}

fn default_redis_required() -> bool {
    false
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
            public_origin: None,
            allowed_origins: Vec::new(),
            cors_supports_credentials: default_cors_supports_credentials(),
            cross_origin_cookies: false,
            trusted_proxies: Vec::new(),
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
        if let Ok(creds) = env::var("CORS_SUPPORTS_CREDENTIALS") {
            builder = builder.set_override(
                "server.cors_supports_credentials",
                creds.to_lowercase() == "true",
            )?;
        }
        if let Ok(cross) = env::var("CROSS_ORIGIN_COOKIES") {
            builder = builder.set_override(
                "server.cross_origin_cookies",
                cross.to_lowercase() == "true",
            )?;
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
        if let Ok(addr) = env::var("REDIS_ADDR") {
            builder = builder.set_override("redis.addr", addr)?;
        }
        // Preferred long-term alias for REDIS_ADDR.
        if let Ok(url) = env::var("REDIS_URL") {
            builder = builder.set_override("redis.addr", url)?;
        }
        if let Ok(ttl) = env::var("ACTIVE_GAME_TTL_SECONDS") {
            if let Ok(ttl) = ttl.parse::<u64>() {
                builder = builder.set_override("redis.ttl_seconds", ttl)?;
            }
        }
        if let Ok(interval) = env::var("SNAPSHOT_INTERVAL_SECONDS") {
            if let Ok(interval) = interval.parse::<u64>() {
                builder = builder.set_override("redis.snapshot_interval_seconds", interval)?;
            }
        }
        if let Ok(required) = env::var("REDIS_REQUIRED_FOR_WRITES") {
            builder = builder.set_override(
                "redis.required_for_writes",
                required.to_lowercase() == "true",
            )?;
        }
        if let Ok(required) = env::var("REDIS_REQUIRED") {
            builder = builder.set_override("redis.required", required.to_lowercase() == "true")?;
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

        let is_production = self.environment.trim().eq_ignore_ascii_case("production");
        if is_production && !self.server.secure_cookies {
            return Err("SECURE_COOKIES must be true in production".into());
        }

        if let Some(origin) = self.server.public_origin.as_ref() {
            let origin = origin.trim();
            if origin.is_empty() {
                return Err("SERVER__PUBLIC_ORIGIN cannot be empty when set".into());
            }
            if is_production && !origin.starts_with("https://") {
                return Err(format!(
                    "SERVER__PUBLIC_ORIGIN must start with https:// in production, got: {origin}"
                ));
            }
            if !(origin.starts_with("https://") || origin.starts_with("http://")) {
                return Err(format!(
                    "SERVER__PUBLIC_ORIGIN must be a full origin (http(s)://...), got: {origin}"
                ));
            }
        }

        // Validate CORS configuration.
        if self.server.cors_supports_credentials
            && is_production
            && self.server.allowed_origins.is_empty()
        {
            return Err(
                "ALLOWED_ORIGINS must be set in production when credentialed CORS is enabled"
                    .into(),
            );
        }

        for origin in &self.server.allowed_origins {
            let origin = origin.trim();
            if self.server.cors_supports_credentials && origin == "*" {
                return Err(
                    "ALLOWED_ORIGINS cannot include '*' when using credentialed CORS".into(),
                );
            }
            if !(origin.starts_with("https://") || origin.starts_with("http://")) {
                return Err(format!(
                    "ALLOWED_ORIGINS must be full origins (http(s)://...), got: {origin}"
                ));
            }
            if is_production && self.server.cross_origin_cookies && !origin.starts_with("https://")
            {
                return Err(format!(
                    "ALLOWED_ORIGINS must start with https:// in production when using cross-origin cookies, got: {origin}"
                ));
            }
        }

        // Cross-origin cookies require explicit origins and secure cookies.
        if self.server.cross_origin_cookies {
            if !self.server.cors_supports_credentials {
                return Err(
                    "CORS_SUPPORTS_CREDENTIALS must be true when using cross-origin cookies".into(),
                );
            }
            if !self.server.secure_cookies {
                return Err("SECURE_COOKIES must be true when using cross-origin cookies".into());
            }
            if self.server.allowed_origins.is_empty() {
                return Err("ALLOWED_ORIGINS must be set when using cross-origin cookies".into());
            }
        }

        // Google OAuth is optional: enable it by providing BOTH client id + secret.
        // If only one is provided, fail closed to avoid a partially configured auth setup.
        let google_id_empty = self.auth.google_client_id.trim().is_empty();
        let google_secret_empty = self.auth.google_client_secret.trim().is_empty();
        match (google_id_empty, google_secret_empty) {
            (true, true) => {}
            (false, false) => {}
            _ => {
                return Err(
                    "GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must both be set (or both be empty to disable Google auth)"
                        .into(),
                );
            }
        }

        // Always require a stable, explicit OAuth callback origin when OAuth is enabled.
        // Avoid constructing it from Host / forwarded headers.
        if !google_id_empty && !google_secret_empty {
            let has_redirect_uri = self
                .auth
                .google_redirect_uri
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            let has_public_origin = self
                .server
                .public_origin
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);

            if !has_redirect_uri && !has_public_origin {
                return Err(
                    "When Google auth is enabled, set GOOGLE_REDIRECT_URI or SERVER__PUBLIC_ORIGIN"
                        .into(),
                );
            }

            if is_production {
                if let Some(uri) = self.auth.google_redirect_uri.as_ref().map(|s| s.trim()) {
                    if !uri.is_empty() && !uri.starts_with("https://") {
                        return Err(format!(
                            "GOOGLE_REDIRECT_URI must start with https:// in production, got: {uri}"
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
