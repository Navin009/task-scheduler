use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

use crate::security::jwt::JWTAuthenticator;

// Define the structure of your claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

// Define a struct for the JWT guard
#[derive(Debug, Clone)]
pub struct JwtAuth {
    pub principal: Claims,
    pub customer_no: String,
}

// Implement FromRequest for JwtAuth to create the guard
#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtAuth {
    type Error = std::io::Error;
    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // Retrieve the Authorization header
        if let Some(authorization) = req.headers().get_one("Authorization") {
            // The token should be in the format "Bearer <JWT>"
            if authorization.starts_with("Bearer ") {
                let authenticator = match req.rocket().state::<JWTAuthenticator>() {
                    Some(authenticator) => authenticator,
                    None => {
                        return Outcome::Error((
                            Status::InternalServerError,
                            Error::new(ErrorKind::Other, "Failed to retrieve app config"),
                        ));
                    }
                };

                let token = &authorization[7..]; // Remove "Bearer " prefix
                match authenticator.validate_jwt(token) {
                    Ok(claims) => Outcome::Success(JwtAuth {
                        customer_no: claims.sub.clone(),
                        principal: claims,
                    }),
                    Err(_) => Outcome::Error((
                        Status::Unauthorized,
                        Error::new(ErrorKind::Other, "Invalid JWT"),
                    )),
                }
            } else {
                Outcome::Error((
                    Status::Unauthorized,
                    Error::new(ErrorKind::Other, "Invalid token format"),
                ))
            }
        } else {
            Outcome::Error((
                Status::Unauthorized,
                Error::new(ErrorKind::Other, "Missing authorization header"),
            ))
        }
    }
}
