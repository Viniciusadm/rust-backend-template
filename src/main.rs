use std::net::SocketAddr;
use std::time::Duration;

use {{crate_name}}::api::rate_limit::RateLimiter;
use {{crate_name}}::{api, config::AppConfig, database};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dotenv_result = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match dotenv_result {
        Ok(path) => info!("loaded environment from {}", path.display()),
        Err(err) if err.not_found() => {}
        Err(err) => warn!("failed to load .env file: {err}"),
    }

    let config = AppConfig::from_env()?;
    let db = if let Some(database_config) = &config.database {
        let pool = database::pool::connect(database_config).await?;
        if config.run_migrations {
            info!("running database migrations");
            sqlx::migrate!("./migrations").run(&pool).await?;
        }
        Some(pool)
    } else {
        warn!(
            "DB_DATABASE not configured; API endpoints that require persistence will return configuration errors"
        );
        None
    };

    let rate_limiter = RateLimiter::new(
        config.auth_rate_limit_max,
        Duration::from_secs(config.auth_rate_limit_window_seconds),
    );
    let state =
        api::state::AppState::new(db, config.jwt_secret.clone()).with_rate_limiter(rate_limiter);
    let app = api::router(state, &config.cors_origins);
    let listener = TcpListener::bind(config.http_addr()).await?;

    info!("backend listening on {}", config.http_addr());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    info!("shutdown signal received");
}
