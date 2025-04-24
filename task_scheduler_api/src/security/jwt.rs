use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::guard::jwt_auth::Claims;

#[derive(Debug)]
pub struct JWTAuthenticator {
    jwt_secret: String,
}

impl JWTAuthenticator {
    pub fn new() -> Self {
        JWTAuthenticator {
            jwt_secret: "secret".to_string(), // In real apps, use a more secure secret
        }
    }

    pub fn create_jwt(&self, username: &str) -> String {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour expiration

        let claims = Claims {
            sub: username.to_string(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .unwrap()
    }

    pub fn validate_jwt(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::HS256))
            .map(|data| data.claims)
    }
}
