use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl Error {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::DatabaseError(_) | Error::RedisError(_) | Error::SerializationError(_)
        )
    }

    pub fn to_http_status(&self) -> u16 {
        match self {
            Error::NotFound(_) => 404,
            Error::BadRequest(_) => 400,
            Error::ValidationError(_) => 400,
            Error::AuthError(_) => 401,
            Error::DatabaseError(_) => 500,
            Error::RedisError(_) => 500,
            Error::ConfigError(_) => 500,
            Error::SerializationError(_) => 500,
            Error::MigrationError(_) => 500,
            Error::InternalServerError(_) => 500,
        }
    }
}
