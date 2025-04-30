use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

use crate::security::jwt::JWTAuthenticator;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JwtAuth {
    pub principal: Claims,
    pub customer_no: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtAuth {
    type Error = std::io::Error;
    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(authorization) = req.headers().get_one("Authorization") {
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

                let token = &authorization[7..];
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
