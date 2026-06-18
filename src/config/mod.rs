use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: Option<String>,
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
        let database_url = env::var("DATABASE_URL").ok();
        let jwt_secret =
            env::var("JWT_SECRET").unwrap_or_else(|_| "development-only-secret".to_owned());
        let run_migrations = env::var("RUN_MIGRATIONS")
            .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "no" | "NO"))
            .unwrap_or(true);

        Self {
            host,
            port,
            database_url,
            jwt_secret,
            run_migrations,
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
