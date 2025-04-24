use crate::config::AppConfig;
use crate::error::ApiError;
use rocket::State;
use rocket::delete;
use rocket::get;
use rocket::post;
use rocket::put;
use rocket::serde::json::Json;
use scheduler_core::api_models::{DeleteResponse, JobCreate, JobResponse, JobUpdate};
use scheduler_core::models::{Job, JobStatus};
use uuid::Uuid;

#[post("/jobs", format = "json", data = "<job>")]
pub async fn create_job(
    state: &State<AppConfig>,
    job: Json<JobCreate>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = job.into_inner();

    let new_job = Job {
        id: Uuid::new_v4().to_string(),
        schedule_type: job.schedule_type,
        schedule: job.schedule,
        payload: job.payload,
        status: JobStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        retries: 0,
        max_retries: job.max_retries as i32,
    };

    state.scheduler_db.create_job(&new_job).await?;

    Ok(Json(JobResponse {
        message: "Job created successfully".to_string(),
        job_id: new_job.id,
    }))
}

#[get("/jobs/<id>")]
pub async fn get_job(state: &State<AppConfig>, id: String) -> Result<Json<Job>, ApiError> {
    match state.scheduler_db.get_job(&id).await? {
        Some(job) => Ok(Json(job)),
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}

#[get("/jobs")]
pub async fn list_jobs(state: &State<AppConfig>) -> Result<Json<Vec<Job>>, ApiError> {
    let jobs = state.scheduler_db.get_pending_jobs(100).await?;
    Ok(Json(jobs))
}

#[put("/jobs/<id>", format = "json", data = "<job>")]
pub async fn update_job(
    state: &State<AppConfig>,
    id: String,
    job: Json<JobUpdate>,
) -> Result<Json<Job>, ApiError> {
    let job_update = job.into_inner();

    match state
        .scheduler_db
        .update_job(
            &id,
            job_update.schedule_type,
            job_update.schedule.as_deref(),
            job_update.payload.as_ref(),
            job_update.max_retries.map(|r| r as i32),
        )
        .await?
    {
        Some(updated_job) => Ok(Json(updated_job)),
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}

#[delete("/jobs/<id>")]
pub async fn delete_job(
    state: &State<AppConfig>,
    id: String,
) -> Result<Json<DeleteResponse>, ApiError> {
    state.scheduler_db.delete_job(&id).await?;

    Ok(Json(DeleteResponse {
        message: format!("Job with id {} deleted successfully", id),
    }))
}
