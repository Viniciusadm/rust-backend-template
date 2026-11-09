use crate::application::auth::{
    AuthenticatedSession, LoginCommand, PasswordHasher, RegisterCommand, RegisterUserRecord,
    TokenIssuer, UserRepository,
};
use crate::domain::{Email, NonEmptyString};
use crate::errors::AppError;
use uuid::Uuid;

pub struct AuthUseCases<R, H, T> {
    users: R,
    password_hasher: H,
    token_issuer: T,
}

impl<R, H, T> AuthUseCases<R, H, T>
where
    R: UserRepository,
    H: PasswordHasher,
    T: TokenIssuer,
{
    pub fn new(users: R, password_hasher: H, token_issuer: T) -> Self {
        Self {
            users,
            password_hasher,
            token_issuer,
        }
    }

    pub async fn login(&self, command: LoginCommand) -> Result<AuthenticatedSession, AppError> {
        let Some(user) = self.users.find_by_email(&command.email).await? else {
            self.password_hasher
                .verify_against_placeholder(&command.password)
                .await;
            return Err(invalid_credentials());
        };

        let valid_password = self
            .password_hasher
            .verify_password(&command.password, user.password_hash.as_str())
            .await?;
        if !valid_password || !user.is_active {
            return Err(invalid_credentials());
        }

        self.users.record_login(user.id.as_str()).await?;

        Ok(AuthenticatedSession {
            access_token: self.token_issuer.issue_access_token(user.id.as_str())?,
            user_id: user.id.as_str().to_owned(),
            display_name: user.display_name.as_str().to_owned(),
        })
    }

    pub async fn register(
        &self,
        command: RegisterCommand,
    ) -> Result<AuthenticatedSession, AppError> {
        let display_name = NonEmptyString::new(command.display_name, "display_name")?
            .as_str()
            .to_owned();
        let email = Email::new(command.email)?.as_str().to_owned();
        if !(8..=128).contains(&command.password.chars().count()) {
            return Err(AppError::Validation(
                "password must be between 8 and 128 characters".to_owned(),
            ));
        }
        if self.users.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("email is already registered".to_owned()));
        }

        let user_id = new_id();
        self.users
            .register_user(RegisterUserRecord {
                user_id: user_id.clone(),
                email,
                password_hash: self
                    .password_hasher
                    .hash_password(&command.password)
                    .await?,
                display_name: display_name.clone(),
            })
            .await?;

        Ok(AuthenticatedSession {
            access_token: self.token_issuer.issue_access_token(&user_id)?,
            user_id,
            display_name,
        })
    }
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized("invalid credentials".to_owned())
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}
