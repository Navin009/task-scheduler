use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Process execution error: {0}")]
    Process(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("State transition error: {0}")]
    StateTransition(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
