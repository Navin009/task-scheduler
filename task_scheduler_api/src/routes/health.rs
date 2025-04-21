use rocket::get;
use rocket::serde::json::Json;
use serde_json::json;

#[get("/health")]
pub fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
} 