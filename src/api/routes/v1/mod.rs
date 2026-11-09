pub mod auth;

use axum::Router;

use crate::api::state::AppState;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new().merge(auth::router(state))
}
