use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueuePopulatorError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Scheduler error: {0}")]
    Scheduler(#[from] scheduler_core::error::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Job processing error: {0}")]
    JobProcessing(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, QueuePopulatorError>;
