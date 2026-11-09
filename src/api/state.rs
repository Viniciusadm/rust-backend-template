use sqlx::MySqlPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Option<MySqlPool>,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db: Option<MySqlPool>, jwt_secret: impl Into<String>) -> Self {
        Self {
            db,
            jwt_secret: jwt_secret.into(),
        }
    }

    pub fn db(&self) -> Result<MySqlPool, crate::errors::AppError> {
        self.db.clone().ok_or_else(|| {
            crate::errors::AppError::Configuration(
                "database is not configured (set DB_DATABASE)".to_owned(),
            )
        })
    }
}
