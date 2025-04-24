use crate::config::AppConfig;
use crate::error::ApiError;
use chrono::{DateTime, Utc};
use rocket::State;
use rocket::delete;
use rocket::get;
use rocket::post;
use rocket::put;
use rocket::serde::json::Json;
use scheduler_core::api_models::{DeleteResponse, JobCreate, JobResponse, JobUpdate};
use scheduler_core::models::{Job, ScheduleType};
use scheduler_core::task::{JobStatus, JobType};
use std::collections::HashMap;
use uuid::Uuid;

#[post("/jobs", format = "json", data = "<job>")]
pub async fn create_job(
    state: &State<AppConfig>,
    job: Json<JobCreate>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = job.into_inner();
    let job_id = Uuid::new_v4().to_string();

    match job.schedule_type {
        ScheduleType::OneTime => {
            // Validate and parse the scheduled time
            let scheduled_at = DateTime::parse_from_rfc3339(&job.schedule).map_err(|_| {
                ApiError::ValidationError("Invalid datetime format. Use ISO 8601 format".into())
            })?;

            state
                .task_manager
                .create_one_time_job(scheduled_at.to_rfc3339(), job.priority as i32, job.payload)
                .await?;
        }
        ScheduleType::Recurring => {
            // Validate cron expression
            let now = Utc::now();
            cron_parser::parse(&job.schedule, &now)
                .map_err(|e| ApiError::ValidationError(e.to_string()))?;

            state
                .task_manager
                .create_recurring_job(
                    job_id.clone(),
                    job.schedule,
                    job.priority as i32,
                    job.payload,
                )
                .await?;
        }
        ScheduleType::Polling => {
            // Parse and validate polling config
            let polling_config: serde_json::Value = serde_json::from_str(&job.schedule)
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
                .create_polling_job(
                    job.schedule,
                    job.priority as i32,
                    max_attempts as i32,
                    job.payload,
                )
                .await?;
        }
    }

    Ok(Json(JobResponse {
        message: "Job created successfully".to_string(),
        job_id,
    }))
}

#[get("/jobs/<id>")]
pub async fn get_job(state: &State<AppConfig>, id: String) -> Result<Json<Job>, ApiError> {
    match state.task_manager.get_job(&id).await? {
        Some(job) => Ok(Json(job)),
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}

#[get("/jobs")]
pub async fn list_jobs(state: &State<AppConfig>) -> Result<Json<Vec<Job>>, ApiError> {
    let jobs = state.task_manager.get_due_jobs(100).await?;
    Ok(Json(jobs))
}

#[put("/jobs/<id>", format = "json", data = "<job>")]
pub async fn update_job(
    state: &State<AppConfig>,
    id: String,
    job: Json<JobUpdate>,
) -> Result<Json<Job>, ApiError> {
    let job_update = job.into_inner();

    match state.task_manager.get_job(&id).await? {
        Some(existing_job) => {
            if let Some(status) = job_update.status {
                state.task_manager.update_job_status(&id, status).await?;
            }

            if let Some(attempts) = job_update.attempts {
                for _ in 0..attempts {
                    state.task_manager.increment_job_attempts(&id).await?;
                }
            }

            match state.task_manager.get_job(&id).await? {
                Some(updated_job) => Ok(Json(updated_job)),
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
    match state.task_manager.get_job(&id).await? {
        Some(_) => {
            state
                .task_manager
                .update_job_status(&id, JobStatus::Failed)
                .await?;
            Ok(Json(DeleteResponse {
                message: format!("Job {} deleted successfully", id),
            }))
        }
        None => Err(ApiError::NotFound(format!("Job with id {} not found", id))),
    }
}
