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
                .scheduler_db
                .create_one_time_job(
                    &job_id,
                    &job.payload,
                    scheduled_at.with_timezone(&Utc),
                    job.max_retries as i32,
                )
                .await?;
        }
        ScheduleType::Recurring => {
            // Validate cron expression
            let now = Utc::now();
            cron_parser::parse(&job.schedule, &now)
                .map_err(|e| ApiError::ValidationError(e.to_string()))?;

            state
                .scheduler_db
                .create_recurring_job(&job_id, &job.payload, &job.schedule, job.max_retries as i32)
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
                .scheduler_db
                .create_polling_job(&job_id, &job.payload, interval as i32, max_attempts as i32)
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

    match state.scheduler_db.get_job(&id).await? {
        Some(existing_job) => match existing_job.schedule_type {
            ScheduleType::OneTime => {
                if let Some(schedule) = job_update.schedule {
                    let scheduled_at = DateTime::parse_from_rfc3339(&schedule).map_err(|_| {
                        ApiError::ValidationError(
                            "Invalid datetime format. Use ISO 8601 format".into(),
                        )
                    })?;

                    match state
                        .scheduler_db
                        .update_job(
                            &id,
                            None,
                            Some(&schedule),
                            job_update.payload.as_ref(),
                            job_update.max_retries.map(|r| r as i32),
                        )
                        .await
                        .map_err(ApiError::from)
                    {
                        Ok(Some(job)) => Ok(Json(job)),
                        Ok(None) => {
                            Err(ApiError::NotFound(format!("Job with id {} not found", id)))
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    match state
                        .scheduler_db
                        .update_job(
                            &id,
                            None,
                            None,
                            job_update.payload.as_ref(),
                            job_update.max_retries.map(|r| r as i32),
                        )
                        .await
                        .map_err(ApiError::from)
                    {
                        Ok(Some(job)) => Ok(Json(job)),
                        Ok(None) => {
                            Err(ApiError::NotFound(format!("Job with id {} not found", id)))
                        }
                        Err(e) => Err(e),
                    }
                }
            }
            ScheduleType::Recurring => {
                if let Some(schedule) = job_update.schedule {
                    let now = Utc::now();
                    cron_parser::parse(&schedule, &now)
                        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

                    match state
                        .scheduler_db
                        .update_job(
                            &id,
                            None,
                            Some(&schedule),
                            job_update.payload.as_ref(),
                            job_update.max_retries.map(|r| r as i32),
                        )
                        .await
                        .map_err(ApiError::from)
                    {
                        Ok(Some(job)) => Ok(Json(job)),
                        Ok(None) => {
                            Err(ApiError::NotFound(format!("Job with id {} not found", id)))
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    match state
                        .scheduler_db
                        .update_job(
                            &id,
                            None,
                            None,
                            job_update.payload.as_ref(),
                            job_update.max_retries.map(|r| r as i32),
                        )
                        .await
                        .map_err(ApiError::from)
                    {
                        Ok(Some(job)) => Ok(Json(job)),
                        Ok(None) => {
                            Err(ApiError::NotFound(format!("Job with id {} not found", id)))
                        }
                        Err(e) => Err(e),
                    }
                }
            }
            ScheduleType::Polling => {
                if let Some(schedule) = job_update.schedule {
                    let polling_config: serde_json::Value = serde_json::from_str(&schedule)
                        .map_err(|_| {
                            ApiError::ValidationError("Invalid polling config format".into())
                        })?;

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

                    match state
                        .scheduler_db
                        .update_job(
                            &id,
                            None,
                            Some(&schedule),
                            job_update.payload.as_ref(),
                            job_update.max_retries.map(|r| r as i32),
                        )
                        .await
                        .map_err(ApiError::from)
                    {
                        Ok(Some(job)) => Ok(Json(job)),
                        Ok(None) => {
                            Err(ApiError::NotFound(format!("Job with id {} not found", id)))
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    match state
                        .scheduler_db
                        .update_job(
                            &id,
                            None,
                            None,
                            job_update.payload.as_ref(),
                            job_update.max_retries.map(|r| r as i32),
                        )
                        .await
                        .map_err(ApiError::from)
                    {
                        Ok(Some(job)) => Ok(Json(job)),
                        Ok(None) => {
                            Err(ApiError::NotFound(format!("Job with id {} not found", id)))
                        }
                        Err(e) => Err(e),
                    }
                }
            }
        },
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
