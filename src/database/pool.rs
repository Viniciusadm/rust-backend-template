use std::time::Duration;

use sqlx::MySqlPool;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};

use crate::config::DatabaseConfig;

pub async fn connect(config: &DatabaseConfig) -> Result<MySqlPool, sqlx::Error> {
    let options = MySqlConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .database(&config.database)
        .username(&config.username)
        .password(&config.password);

    MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await
}
