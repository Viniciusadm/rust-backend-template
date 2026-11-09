use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::api::state::AppState;
use crate::application::auth::TokenVerifier;
use crate::errors::AppError;
use crate::infrastructure::auth::JwtTokenService;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AppError::Unauthorized("missing bearer token".to_owned()))?;

        let user_id = JwtTokenService::new(state.jwt_secret.clone()).verify_access_token(token)?;

        Ok(Self { user_id })
    }
}
