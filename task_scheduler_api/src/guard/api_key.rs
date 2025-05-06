use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::model::auth::{AuthContext, Merchant, User};

#[derive(Debug)]
pub struct ApiKeyGuard(pub AuthContext);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKeyGuard {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let api_key = request.headers().get_one("X-API-Key");

        if let Some(api_key) = api_key {
            let db = request.rocket().state::<PgPool>().unwrap();

            // Query to get merchant and user details
            let result = sqlx::query!(
                r#"
                SELECT 
                    m.id as merchant_id,
                    m.name as merchant_name,
                    ak.key as api_key,
                    u.id as user_id,
                    u.username as email,
                    u.role
                FROM api_keys ak
                JOIN merchants m ON m.id = ak.merchant_id
                JOIN users u ON u.merchant_id = m.id
                WHERE ak.key = $1 AND ak.active = true AND ak.expires_at > NOW()
                "#,
                api_key
            )
            .fetch_one(db)
            .await;

            match result {
                Ok(record) => {
                    let merchant = Merchant {
                        id: record.merchant_id,
                        name: record.merchant_name,
                        api_key: record.api_key,
                    };

                    let user = User {
                        id: record.user_id,
                        merchant_id: record.merchant_id,
                        email: record.email,
                        role: record.role,
                    };

                    Outcome::Success(ApiKeyGuard(AuthContext::new(merchant, user)))
                }
                Err(_) => Outcome::Error((Status::Unauthorized, ApiError::InvalidApiKey)),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ApiError::MissingApiKey))
        }
    }
}
