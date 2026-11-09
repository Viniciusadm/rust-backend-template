use std::sync::Arc;
use std::sync::OnceLock;

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher as ArgonPasswordHasher, PasswordVerifier};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::application::auth::{PasswordHasher, TokenIssuer, TokenVerifier};
use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let password = password.to_owned();
        run_blocking(move || hash_blocking(&password)).await?
    }

    async fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool, AppError> {
        let password = password.to_owned();
        let password_hash = password_hash.to_owned();
        run_blocking(move || verify_blocking(&password, &password_hash)).await?
    }

    async fn verify_against_placeholder(&self, password: &str) {
        let password = password.to_owned();
        let _ = run_blocking(move || {
            let placeholder = placeholder_hash();
            let _ = verify_blocking(&password, placeholder);
        })
        .await;
    }
}

async fn run_blocking<T, F>(task: F) -> Result<T, AppError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| AppError::Internal(format!("blocking task failed: {error}")))
}

fn hash_blocking(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::Internal(format!("password hash failed: {error}")))
}

fn verify_blocking(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AppError::Unauthorized("invalid credentials".to_owned()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn placeholder_hash() -> &'static str {
    static PLACEHOLDER: OnceLock<Arc<str>> = OnceLock::new();
    PLACEHOLDER.get_or_init(|| {
        hash_blocking("placeholder-password")
            .unwrap_or_default()
            .into()
    })
}

#[derive(Debug, Clone)]
pub struct JwtTokenService {
    secret: String,
}

impl JwtTokenService {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

impl TokenIssuer for JwtTokenService {
    fn issue_access_token(&self, user_id: &str) -> Result<String, AppError> {
        let expires_at = Utc::now() + Duration::hours(12);
        let claims = AccessTokenClaims {
            sub: user_id.to_owned(),
            exp: expires_at.timestamp() as usize,
        };
        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|error| AppError::Internal(format!("jwt issue failed: {error}")))
    }
}

impl TokenVerifier for JwtTokenService {
    fn verify_access_token(&self, token: &str) -> Result<String, AppError> {
        jsonwebtoken::decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims.sub)
        .map_err(|_| AppError::Unauthorized("invalid access token".to_owned()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccessTokenClaims {
    sub: String,
    exp: usize,
}
