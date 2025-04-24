use crate::error::ApiError;
use rocket::delete;
use rocket::get;
use rocket::post;
use rocket::put;
use rocket::serde::json::Json;
use scheduler_core::models::Template;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateCreate {
    pub cron_pattern: String,
    pub payload_template: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateUpdate {
    pub cron_pattern: Option<String>,
    pub payload_template: Option<serde_json::Value>,
    pub active: Option<bool>,
}

#[post("/templates", format = "json", data = "<template>")]
pub async fn create_template(
    template: Json<TemplateCreate>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement template creation using scheduler_core
    Ok(Json(json!({
        "message": "Template created successfully",
        "template": template.into_inner()
    })))
}

#[get("/templates/<id>")]
pub async fn get_template(id: i32) -> Result<Json<Template>, ApiError> {
    // TODO: Implement template retrieval using scheduler_core
    Err(ApiError::NotFound(format!(
        "Template with id {} not found",
        id
    )))
}

#[get("/templates")]
pub async fn list_templates() -> Result<Json<Vec<Template>>, ApiError> {
    // TODO: Implement template listing using scheduler_core
    Ok(Json(Vec::new()))
}

#[put("/templates/<id>", format = "json", data = "<template>")]
pub async fn update_template(
    id: i32,
    template: Json<TemplateUpdate>,
) -> Result<Json<Template>, ApiError> {
    // TODO: Implement template update using scheduler_core
    // For now, just return a not found error
    Err(ApiError::NotFound(format!(
        "Template with id {} not found",
        id
    )))
}

#[delete("/templates/<id>")]
pub async fn delete_template(id: i32) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement template deletion using scheduler_core
    Ok(Json(json!({
        "message": format!("Template with id {} deleted successfully", id)
    })))
}
