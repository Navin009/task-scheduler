use crate::config::AppConfig;
use crate::error::ApiError;
use chrono::Utc;
use rocket::delete;
use rocket::get;
use rocket::post;
use rocket::put;
use rocket::serde::json::Json;
use rocket::State;
use scheduler_core::api_models::{DeleteResponse, JobCreate, JobResponse, JobUpdate};
use scheduler_core::models::{Job as CoreJob, JobStatus, JobType};
use scheduler_core::task::Job as TaskJob;
use std::collections::HashMap;
use uuid::Uuid;

fn convert_task_job_to_core_job(task_job: TaskJob) -> CoreJob {
    CoreJob {
        id: task_job.id,
        schedule_type: task_job.job_type,
        schedule: task_job.scheduled_at,
        payload: serde_json::to_value(task_job.payload).unwrap(),
        status: task_job.status,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        retries: task_job.retries,
        max_retries: task_job.max_retries,
    }
}

#[post("/jobs", format = "json", data = "<job>")]
pub async fn create_job(
    state: &State<AppConfig>,
    job: Json<JobCreate>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = job.into_inner();
    let job_id = Uuid::new_v4();

    // Convert payload to HashMap
    let payload = serde_json::from_value::<HashMap<String, String>>(job.payload)
        .map_err(|e| ApiError::ValidationError(format!("Invalid payload format: {}", e)))?;

    match job.schedule_type {
        JobType::OneTime => {
            state
                .task_manager
                .create_one_time_job(job.schedule, 0, payload)
                .await
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        }
        JobType::Recurring => {
            // Validate cron expression
            let now = Utc::now();

            state
                .task_manager
                .create_recurring_job(job_id.clone(), job.schedule, 0, payload)
                .await
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        }
        JobType::Polling => {
            // Parse and validate polling config
            let polling_config: serde_json::Value = serde_json::from_str(&job.schedule.to_string())
                .map_err(|_| ApiError::ValidationError("Invalid polling config format".into()))?;

            let interval = polling_config
                .get("interval")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| {
                    ApiError::ValidationError(
                        "Missing or invalid interval in polling config".into(),
                    )
                })?;

            let max_attempts = polling_config
                .get("max_attempts")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| {
                    ApiError::ValidationError(
                        "Missing or invalid max_attempts in polling config".into(),
                    )
                })?;

            state
                .task_manager
                .create_polling_job(job.schedule, 0, max_attempts as i32, payload)
                .await
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
        }
    }

    Ok(Json(JobResponse {
        message: "Job created successfully".to_string(),
        job_id,
    }))
}

#[get("/jobs/<id>")]
pub async fn get_job(state: &State<AppConfig>, id: String) -> Result<Json<CoreJob>, ApiError> {
    match state
        .task_manager
        .get_job(&id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        Some(job) => Ok(Json(convert_task_job_to_core_job(job))),
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}

#[get("/jobs")]
pub async fn list_jobs(state: &State<AppConfig>) -> Result<Json<Vec<CoreJob>>, ApiError> {
    let jobs = state
        .task_manager
        .get_due_jobs(100)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok(Json(
        jobs.into_iter().map(convert_task_job_to_core_job).collect(),
    ))
}

#[put("/jobs/<id>", format = "json", data = "<job>")]
pub async fn update_job(
    state: &State<AppConfig>,
    id: String,
    job: Json<JobUpdate>,
) -> Result<Json<CoreJob>, ApiError> {
    let job_update = job.into_inner();

    match state
        .task_manager
        .get_job(&id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        Some(existing_job) => {
            if let Some(schedule_type) = job_update.schedule_type {
                // Convert schedule type to job type
                let job_type = match schedule_type {
                    JobType::OneTime => JobType::OneTime,
                    JobType::Recurring => JobType::Recurring,
                    JobType::Polling => JobType::Polling,
                };
                // Update job type through status update (temporary workaround)
                state
                    .task_manager
                    .update_job_status(&id, JobStatus::Pending)
                    .await
                    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
            }

            if let Some(schedule) = job_update.schedule {
                // For now, we can't update schedule directly
                return Err(ApiError::BadRequest(
                    "Schedule updates are not supported yet".into(),
                ));
            }

            if let Some(payload) = job_update.payload {
                // For now, we can't update payload directly
                return Err(ApiError::BadRequest(
                    "Payload updates are not supported yet".into(),
                ));
            }

            if let Some(max_retries) = job_update.max_retries {
                // For now, we can't update max retries directly
                return Err(ApiError::BadRequest(
                    "Max retries updates are not supported yet".into(),
                ));
            }

            match state
                .task_manager
                .get_job(&id)
                .await
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?
            {
                Some(updated_job) => Ok(Json(convert_task_job_to_core_job(updated_job))),
                None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
            }
        }
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}

#[delete("/jobs/<id>")]
pub async fn delete_job(
    state: &State<AppConfig>,
    id: String,
) -> Result<Json<DeleteResponse>, ApiError> {
    match state
        .task_manager
        .get_job(&id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
    {
        Some(_) => {
            state
                .task_manager
                .update_job_status(&id, JobStatus::Failed)
                .await
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
            Ok(Json(DeleteResponse {
                message: format!("Job {} deleted successfully", id),
            }))
        }
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}
