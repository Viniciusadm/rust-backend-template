use std::env;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Option<Self> {
        let database = env::var("DB_DATABASE")
            .ok()
            .filter(|value| !value.is_empty())?;
        let host = env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("DB_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3306);
        let username = env::var("DB_USERNAME").unwrap_or_else(|_| "root".to_owned());
        let password = env::var("DB_PASSWORD").unwrap_or_default();

        Some(Self {
            host,
            port,
            database,
            username,
            password,
        })
    }
}

const DEVELOPMENT_JWT_SECRET: &str = "development-only-secret";
const MIN_PRODUCTION_JWT_SECRET_LENGTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid configuration: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database: Option<DatabaseConfig>,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub run_migrations: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("HTTP_HOST")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let port = env::var("HTTP_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3001);
        let database = DatabaseConfig::from_env();
        let is_production = env::var("APP_ENV")
            .map(|value| value.eq_ignore_ascii_case("production"))
            .unwrap_or(false);
        let jwt_secret = resolve_jwt_secret(env::var("JWT_SECRET").ok(), is_production)?;
        let cors_origins = parse_origins(&env::var("CORS_ORIGINS").unwrap_or_default());
        let run_migrations = env::var("RUN_MIGRATIONS")
            .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "no" | "NO"))
            .unwrap_or(true);

        Ok(Self {
            host,
            port,
            database,
            jwt_secret,
            cors_origins,
            run_migrations,
        })
    }

    pub fn http_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

fn resolve_jwt_secret(value: Option<String>, is_production: bool) -> Result<String, ConfigError> {
    if !is_production {
        return Ok(value.unwrap_or_else(|| DEVELOPMENT_JWT_SECRET.to_owned()));
    }
    match value {
        Some(secret)
            if secret.len() >= MIN_PRODUCTION_JWT_SECRET_LENGTH
                && secret != DEVELOPMENT_JWT_SECRET =>
        {
            Ok(secret)
        }
        _ => Err(ConfigError(format!(
            "JWT_SECRET must be set to a custom value with at least {MIN_PRODUCTION_JWT_SECRET_LENGTH} characters when APP_ENV=production"
        ))),
    }
}

fn parse_origins(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_falls_back_to_default_secret() {
        assert_eq!(
            resolve_jwt_secret(None, false).unwrap(),
            DEVELOPMENT_JWT_SECRET
        );
    }

    #[test]
    fn production_requires_strong_custom_secret() {
        assert!(resolve_jwt_secret(None, true).is_err());
        assert!(resolve_jwt_secret(Some("short".to_owned()), true).is_err());
        assert!(resolve_jwt_secret(Some(DEVELOPMENT_JWT_SECRET.to_owned()), true).is_err());
        assert!(resolve_jwt_secret(Some("x".repeat(32)), true).is_ok());
    }

    #[test]
    fn parses_comma_separated_origins() {
        assert_eq!(
            parse_origins(" https://a.com, https://b.com ,,"),
            vec!["https://a.com", "https://b.com"]
        );
        assert!(parse_origins("").is_empty());
    }
}
