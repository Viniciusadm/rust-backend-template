pub mod dto;
pub mod extractors;
pub mod routes;
pub mod state;

use axum::Router;
use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use state::AppState;

pub fn router(state: AppState, cors_origins: &[String]) -> Router {
    Router::<AppState>::new()
        .merge(routes::health::router())
        .nest("/api/v1", routes::v1::router())
        .with_state(state)
        .layer(cors_layer(cors_origins))
        .layer(TraceLayer::new_for_http())
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    if origins.is_empty() {
        return layer.allow_origin(Any);
    }
    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();
    layer.allow_origin(allowed)
}
