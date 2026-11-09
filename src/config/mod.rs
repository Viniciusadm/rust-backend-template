use std::env;
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
        let database = env::var("DB_DATABASE").ok().filter(|value| !value.is_empty())?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database: Option<DatabaseConfig>,
    pub jwt_secret: String,
    pub run_migrations: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("HTTP_HOST")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let port = env::var("HTTP_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3001);
        let database = DatabaseConfig::from_env();
        let jwt_secret =
            env::var("JWT_SECRET").unwrap_or_else(|_| "development-only-secret".to_owned());
        let run_migrations = env::var("RUN_MIGRATIONS")
            .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "no" | "NO"))
            .unwrap_or(true);

        Self {
            host,
            port,
            database,
            jwt_secret,
            run_migrations,
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
