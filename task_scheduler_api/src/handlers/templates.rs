use crate::config::AppConfig;
use crate::error::ApiError;
use rocket::State;
use rocket::delete;
use rocket::get;
use rocket::post;
use rocket::put;
use rocket::serde::json::Json;
use scheduler_core::api_models::{
    DeleteResponse, TemplateCreate, TemplateResponse, TemplateUpdate,
};
use scheduler_core::models::Template;

#[post("/templates", format = "json", data = "<template>")]
pub async fn create_template(
    state: &State<AppConfig>,
    template: Json<TemplateCreate>,
) -> Result<Json<TemplateResponse>, ApiError> {
    let template = template.into_inner();

    // TODO: Implement template creation using the actual Database interface
    Err(ApiError::InternalServerError(
        "Template creation not implemented yet".to_string(),
    ))
}

#[get("/templates/<id>")]
pub async fn get_template(state: &State<AppConfig>, id: i32) -> Result<Json<Template>, ApiError> {
    // TODO: Implement template retrieval using scheduler_core
    Err(ApiError::NotFound(format!(
        "Template with id {} not found",
        id
    )))
}

#[get("/templates")]
pub async fn list_templates(state: &State<AppConfig>) -> Result<Json<Vec<Template>>, ApiError> {
    // TODO: Implement template listing using scheduler_core
    Ok(Json(Vec::new()))
}

#[put("/templates/<id>", format = "json", data = "<template>")]
pub async fn update_template(
    state: &State<AppConfig>,
    id: i32,
    template: Json<TemplateUpdate>,
) -> Result<Json<Template>, ApiError> {
    // TODO: Implement template update using scheduler_core
    Err(ApiError::NotFound(format!(
        "Template with id {} not found",
        id
    )))
}

#[delete("/templates/<id>")]
pub async fn delete_template(
    state: &State<AppConfig>,
    id: i32,
) -> Result<Json<DeleteResponse>, ApiError> {
    // TODO: Implement template deletion using scheduler_core
    Ok(Json(DeleteResponse {
        message: format!("Template with id {} deleted successfully", id),
    }))
}
