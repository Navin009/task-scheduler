use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueuePopulatorError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error: {0}")]
    Cache(#[from] scheduler_core::error::CacheError),

    #[error("Configuration error: {0}")]
    Config(#[from] scheduler_core::error::ConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Job processing error: {0}")]
    JobProcessing(String),
}

pub type Result<T> = std::result::Result<T, QueuePopulatorError>;
