use axum::{Router, routing::get};

use crate::api::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> &'static str {
    "ok"
}
