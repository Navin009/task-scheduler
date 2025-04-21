use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::Request;
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    DatabaseError(String),
    RedisError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            ApiError::RedisError(msg) => write!(f, "Redis Error: {}", msg),
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
        };

        let body = json!({
            "error": message,
            "status": status.code
        });

        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(body.to_string().len(), std::io::Cursor::new(body.to_string()))
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