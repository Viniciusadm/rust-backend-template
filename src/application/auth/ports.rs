use async_trait::async_trait;

use crate::domain::users::User;
use crate::errors::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterUserRecord {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn register_user(&self, record: RegisterUserRecord) -> Result<(), AppError>;
}

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash_password(&self, password: &str) -> Result<String, AppError>;
    async fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool, AppError>;

    async fn verify_against_placeholder(&self, _password: &str) {}
}

pub trait TokenIssuer: Send + Sync {
    fn issue_access_token(&self, user_id: &str) -> Result<String, AppError>;
}

pub trait TokenVerifier: Send + Sync {
    fn verify_access_token(&self, token: &str) -> Result<String, AppError>;
}
