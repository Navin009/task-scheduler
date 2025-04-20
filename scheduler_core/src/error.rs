use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SchedulerError>;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Invalid schedule format: {0}")]
    InvalidSchedule(String),

    #[error("Task execution error: {0}")]
    TaskExecution(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Lock acquisition failed: {0}")]
    LockAcquisition(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

// Implement conversion from sqlx errors
impl From<sqlx::Error> for SchedulerError {
    fn from(err: sqlx::Error) -> Self {
        SchedulerError::Database(err.to_string())
    }
}

// Implement conversion from redis errors
impl From<redis::RedisError> for SchedulerError {
    fn from(err: redis::RedisError) -> Self {
        SchedulerError::Redis(err.to_string())
    }
}

// Implement conversion from cron-parser errors
impl From<cron::error::Error> for SchedulerError {
    fn from(err: cron::error::Error) -> Self {
        SchedulerError::InvalidSchedule(err.to_string())
    }
}
