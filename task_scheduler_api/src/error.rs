use rocket::catch;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::Request;
use scheduler_core::error::Error as SchedulerError;
use serde_json::json;
use thiserror::Error;

#[allow(dead_code)]
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

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Database Error: {0}")]
    DatabaseError(String),
    #[error("Redis Error: {0}")]
    RedisError(String),
    #[error("Validation Error: {0}")]
    ValidationError(String),
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Invalid API key")]
    InvalidApiKey,
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

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (Status::NotFound, msg),
            ApiError::BadRequest(msg) => (Status::BadRequest, msg),
            ApiError::InternalServerError(msg) => (Status::InternalServerError, msg),
            ApiError::DatabaseError(msg) => (Status::InternalServerError, msg),
            ApiError::RedisError(msg) => (Status::InternalServerError, msg),
            ApiError::ValidationError(msg) => (Status::BadRequest, msg),
            ApiError::MissingApiKey => (Status::BadRequest, "Missing API key".to_string()),
            ApiError::InvalidApiKey => (Status::BadRequest, "Invalid API key".to_string()),
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

#[allow(dead_code)]
#[catch(404)]
pub fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[allow(dead_code)]
#[catch(422)]
pub fn unprocessable_entity() -> String {
    "The request was well-formed but was unable to be followed due to semantic errors.".to_string()
}

#[allow(dead_code)]
#[catch(500)]
pub fn internal_server_error() -> String {
    "Whoops! Looks like we messed up.".to_string()
}
