use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::Request;
use scheduler_core::error::Error as SchedulerError;
use serde_json::json;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    DatabaseError(String),
    RedisError(String),
    ValidationError(String),
}

impl From<SchedulerError> for ApiError {
    fn from(error: SchedulerError) -> Self {
        match error {
            SchedulerError::DatabaseError(e) => ApiError::DatabaseError(e.to_string()),
            SchedulerError::RedisError(e) => ApiError::RedisError(e.to_string()),
            SchedulerError::ConfigError(e) => ApiError::InternalServerError(e),
            SchedulerError::ValidationError(e) => ApiError::ValidationError(e),
            SchedulerError::SerializationError(e) => ApiError::InternalServerError(e.to_string()),
            SchedulerError::MigrationError(e) => ApiError::InternalServerError(e.to_string()),
            SchedulerError::AuthError(e) => ApiError::BadRequest(e),
            SchedulerError::NotFound(e) => ApiError::NotFound(e),
            SchedulerError::BadRequest(e) => ApiError::BadRequest(e),
            SchedulerError::InternalServerError(e) => ApiError::InternalServerError(e),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ApiError::RedisError(msg) => write!(f, "Redis Error: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (Status::NotFound, msg),
            ApiError::BadRequest(msg) => (Status::BadRequest, msg),
            ApiError::InternalServerError(msg) => (Status::InternalServerError, msg),
            ApiError::DatabaseError(msg) => (Status::InternalServerError, msg),
            ApiError::RedisError(msg) => (Status::InternalServerError, msg),
            ApiError::ValidationError(msg) => (Status::BadRequest, msg),
        };

        let body = json!({
            "error": message,
            "status": status.code
        });

        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(
                body.to_string().len(),
                std::io::Cursor::new(body.to_string()),
            )
            .ok()
    }
}

#[catch(404)]
pub fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[catch(422)]
pub fn unprocessable_entity() -> String {
    "The request was well-formed but was unable to be followed due to semantic errors.".to_string()
}

#[catch(500)]
pub fn internal_server_error() -> String {
    "Whoops! Looks like we messed up.".to_string()
}
