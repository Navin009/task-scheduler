use std::time::Duration;

use rocket::{State, serde::json::Json};
use serde::Serialize;
use sqlx::postgres::PgPool;
use tokio::time::timeout;

use crate::config::AppConfig;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
pub struct DbCheckResponse {
    database_connected: bool,
}

#[get("/ping")]
pub async fn ping() -> &'static str {
    "pong"
}

#[get("/health")]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

#[get("/metrics")]
pub async fn metrics() -> String {
    String::from("TODO : Add metrics")
}

#[get("/db-check")]
pub async fn db_check(state: &State<AppConfig>) -> Json<DbCheckResponse> {
    let timeout_duration = Duration::new(5, 0); // 5 seconds
    let result = timeout(
        timeout_duration,
        sqlx::query("SELECT 1").fetch_one(&state.postgres),
    )
    .await;

    let database_connected = match result {
        Ok(Ok(_)) => true,   // Query succeeded within the timeout
        Ok(Err(_)) => false, // Query failed (e.g., PostgreSQL not reachable)
        Err(_) => false,     // Query timed out
    };
    Json(DbCheckResponse { database_connected })
}

#[get("/prometheus")]
pub async fn prometheus() -> String {
    //TODO : Add prometheus metrics
    String::from("TODO : Add prometheus metrics")
}
