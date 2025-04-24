use std::str;

use base64::{Engine, prelude::BASE64_STANDARD};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
};

use crate::config::AppConfig;

#[derive(Debug)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = std::io::Error;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // Attempt to get the AppConfig from the request state
        let app_config: Option<&AppConfig> = req.rocket().state();

        // Check if app_config is available, otherwise return a 500 Internal Server Error
        let app_config = match app_config {
            Some(config) => config,
            None => {
                return Outcome::Error((
                    Status::InternalServerError,
                    std::io::Error::new(std::io::ErrorKind::Other, "Failed to retrieve app config"),
                ));
            }
        };

        // Assuming AppConfig has a hashmap of valid usernames and passwords
        let valid_credentials = match app_config.get_basic_auth() {
            Ok(credentials) => credentials,
            Err(err) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    std::io::Error::new(std::io::ErrorKind::Other, err.to_string()),
                ));
            }
        };

        if let Some(authorization) = req.headers().get_one("Authorization") {
            if authorization.starts_with("Basic ") {
                let encoded = &authorization[6..]; // Remove "Basic " prefix
                match BASE64_STANDARD.decode(encoded) {
                    Ok(decoded_bytes) => {
                        if let Ok(decoded_str) = str::from_utf8(&decoded_bytes) {
                            let parts: Vec<&str> = decoded_str.split(':').collect();
                            if parts.len() == 2 {
                                let username = parts[0].to_string();
                                let password = parts[1].to_string();

                                // Check if the provided username and password match the config

                                if valid_credentials.username == username
                                    && valid_credentials.password == password
                                {
                                    return Outcome::Success(BasicAuth { username, password });
                                }

                                // If the credentials are invalid, return a 401 Unauthorized response
                                return Outcome::Error((
                                    Status::Unauthorized,
                                    std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        "Invalid credentials",
                                    ),
                                ));
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        // If we reach here, the credentials are invalid or missing
        Outcome::Error((
            Status::Unauthorized,
            std::io::Error::new(std::io::ErrorKind::Other, "Invalid credentials"),
        ))
    }
}
