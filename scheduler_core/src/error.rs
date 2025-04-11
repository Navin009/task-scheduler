use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Failed to acquire lock for task: {0}")]
    LockError(Uuid),

    #[error("Invalid schedule format: {0}")]
    InvalidSchedule(String),

    #[error("Initialization failed: {0}")]
    Initialization(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;
