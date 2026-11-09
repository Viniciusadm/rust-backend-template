use sqlx::MySqlPool;

use crate::api::rate_limit::RateLimiter;
use crate::application::auth::AuthUseCases;
use crate::errors::AppError;
use crate::infrastructure::auth::{Argon2PasswordHasher, JwtTokenService};
use crate::infrastructure::repositories::mysql_users::MySqlUserRepository;

pub type DefaultAuthUseCases =
    AuthUseCases<MySqlUserRepository, Argon2PasswordHasher, JwtTokenService>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Option<MySqlPool>,
    pub jwt_secret: String,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(db: Option<MySqlPool>, jwt_secret: impl Into<String>) -> Self {
        Self {
            db,
            jwt_secret: jwt_secret.into(),
            rate_limiter: RateLimiter::disabled(),
        }
    }

    pub fn with_rate_limiter(mut self, rate_limiter: RateLimiter) -> Self {
        self.rate_limiter = rate_limiter;
        self
    }

    pub fn db(&self) -> Result<MySqlPool, AppError> {
        self.db.clone().ok_or_else(|| {
            AppError::Configuration("database is not configured (set DB_DATABASE)".to_owned())
        })
    }

    pub fn user_repository(&self) -> Result<MySqlUserRepository, AppError> {
        Ok(MySqlUserRepository::new(self.db()?))
    }

    pub fn auth_use_cases(&self) -> Result<DefaultAuthUseCases, AppError> {
        Ok(AuthUseCases::new(
            self.user_repository()?,
            Argon2PasswordHasher,
            JwtTokenService::new(self.jwt_secret.clone()),
        ))
    }
}
