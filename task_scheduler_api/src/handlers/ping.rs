use std::time::Duration;

use crate::error::ApiError;
use rocket::get;
use rocket::{State, serde::json::Json};
use serde::Serialize;
use serde_json::json;
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
pub async fn ping(state: &State<AppConfig>) -> Result<Json<serde_json::Value>, ApiError> {
    // Try to execute a simple query to check database connectivity
    match state.task_manager.get_due_jobs(1).await {
        Ok(_) => Ok(Json(json!({
            "status": "ok",
            "message": "Service is healthy and database is connected"
        }))),
        Err(e) => Ok(Json(json!({
            "status": "error",
            "message": format!("Service is healthy but database connection failed: {}", e)
        }))),
    }
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
    let result = timeout(timeout_duration, state.task_manager.get_due_jobs(1)).await;

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
