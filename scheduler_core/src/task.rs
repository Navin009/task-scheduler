use crate::db::Database as Db;
use crate::error::Error;
use crate::models::{Job, JobStatus, ScheduleType, Template};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

/// Creates a new job with the specified parameters and saves it to the database
pub async fn create_job(
    db: &Db,
    schedule_type: ScheduleType,
    schedule: String,
    payload: Value,
    max_retries: u32,
) -> Result<Job, Error> {
    let now = Utc::now();
    let job = Job {
        id: Uuid::new_v4().to_string(),
        schedule_type,
        schedule,
        payload,
        status: JobStatus::Pending,
        created_at: now,
        updated_at: now,
        retries: 0,
        max_retries: max_retries as i32,
    };

    job.validate()?;
    db.create_job(&job).await?;
    Ok(job)
}

/// Creates a new template for recurring jobs and saves it to the database
pub async fn create_template(
    db: &Db,
    cron_pattern: String,
    payload_template: Value,
) -> Result<Template, Error> {
    // Validate cron pattern
    cron_parser::parse(&cron_pattern).map_err(|e| Error::ValidationError(e.to_string()))?;

    let now = Utc::now();
    let template = Template {
        id: Uuid::new_v4().to_string(),
        cron_pattern,
        payload_template,
        active: true,
        created_at: now,
        updated_at: now,
    };

    db.create_template(&template).await?;
    Ok(template)
}

/// Updates the status of a job in the database
pub async fn update_job_status(db: &Db, job: &mut Job, new_status: JobStatus) -> Result<(), Error> {
    job.status = new_status;
    job.updated_at = Utc::now();
    db.update_job_status(&job.id, new_status).await?;
    Ok(())
}

/// Increments the retry count for a job in the database
pub async fn increment_retry_count(db: &Db, job: &mut Job) -> Result<(), Error> {
    if job.retries >= job.max_retries {
        return Err(Error::MaxRetriesExceeded);
    }
    job.retries += 1;
    job.updated_at = Utc::now();
    db.increment_job_retries(&job.id).await?;
    Ok(())
}

/// Calculates the next execution time for a job
pub fn calculate_next_execution(job: &Job) -> Result<DateTime<Utc>, Error> {
    match job.schedule_type {
        ScheduleType::Immediate => Ok(Utc::now()),
        ScheduleType::Cron => {
            let schedule = cron_parser::parse(&job.schedule)
                .map_err(|e| Error::ValidationError(e.to_string()))?;
            schedule
                .next_after(&Utc::now())
                .ok_or_else(|| Error::InvalidSchedule("No future execution time found".into()))
        }
        ScheduleType::Interval => {
            let interval_seconds = job
                .schedule
                .parse::<u64>()
                .map_err(|_| Error::ValidationError("Invalid interval format".into()))?;
            Ok(Utc::now() + Duration::from_secs(interval_seconds))
        }
    }
}

/// Generates a job from a template and saves it to the database
pub async fn generate_job_from_template(db: &Db, template: &Template) -> Result<Job, Error> {
    let job = create_job(
        db,
        ScheduleType::Cron,
        template.cron_pattern.clone(),
        template.payload_template.clone(),
        3, // Default max retries
    )
    .await?;
    Ok(job)
}

/// Validates if a job can be executed
pub fn can_execute_job(job: &Job) -> bool {
    job.status == JobStatus::Pending
        || (job.status == JobStatus::Failed && job.retries < job.max_retries)
}

/// Marks a job as completed in the database
pub async fn mark_job_completed(db: &Db, job: &mut Job) -> Result<(), Error> {
    update_job_status(db, job, JobStatus::Completed).await
}

/// Marks a job as failed in the database
pub async fn mark_job_failed(db: &Db, job: &mut Job) -> Result<(), Error> {
    update_job_status(db, job, JobStatus::Failed).await
}

/// Marks a job as running in the database
pub async fn mark_job_running(db: &Db, job: &mut Job) -> Result<(), Error> {
    update_job_status(db, job, JobStatus::Running).await
}

/// Marks a job as retrying in the database
pub async fn mark_job_retrying(db: &Db, job: &mut Job) -> Result<(), Error> {
    update_job_status(db, job, JobStatus::Retrying).await
}

/// Gets due jobs from the database
pub async fn get_due_jobs(db: &Db, batch_size: i64) -> Result<Vec<Job>, Error> {
    db.get_due_jobs(batch_size).await
}

/// Gets a job by ID from the database
pub async fn get_job_by_id(db: &Db, job_id: &str) -> Result<Job, Error> {
    db.get_job_by_id(job_id).await
}

/// Gets all active templates from the database
pub async fn get_active_templates(db: &Db) -> Result<Vec<Template>, Error> {
    db.get_active_templates().await
}

/// Updates a template in the database
pub async fn update_template(db: &Db, template: &Template) -> Result<(), Error> {
    db.update_template(template).await
}

/// Deletes a job from the database
pub async fn delete_job(db: &Db, job_id: &str) -> Result<(), Error> {
    db.delete_job(job_id).await
}

/// Deletes a template from the database
pub async fn delete_template(db: &Db, template_id: &str) -> Result<(), Error> {
    db.delete_template(template_id).await
}
