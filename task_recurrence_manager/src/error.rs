use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Schedule parsing error: {0}")]
    ScheduleParse(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Validation error: {0}")]
    Validation(String),
}
