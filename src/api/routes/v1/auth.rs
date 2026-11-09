use axum::extract::State;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::api::dto::auth::{LoginRequest, LoginResponse, MeResponse, MeUser, RegisterRequest};
use crate::api::extractors::{ApiJson, AuthenticatedUser};
use crate::api::rate_limit::limit_requests;
use crate::api::state::AppState;
use crate::application::auth::UserRepository;
use crate::errors::AppError;

pub fn router(state: &AppState) -> Router<AppState> {
    let credentials = Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route_layer(from_fn_with_state(state.clone(), limit_requests));

    Router::new().merge(credentials).route("/auth/me", get(me))
}

async fn me(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<MeResponse>, AppError> {
    let user = state
        .user_repository()?
        .find_by_id(&auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user was not found".to_owned()))?;

    Ok(Json(MeResponse {
        user: MeUser {
            id: user.id.as_str().to_owned(),
            display_name: user.display_name.as_str().to_owned(),
            email: user.email.as_str().to_owned(),
        },
    }))
}

async fn login(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let session = state.auth_use_cases()?.login(request.into()).await?;
    Ok(Json(session.into()))
}

async fn register(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<RegisterRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let session = state.auth_use_cases()?.register(request.into()).await?;
    Ok(Json(session.into()))
}
