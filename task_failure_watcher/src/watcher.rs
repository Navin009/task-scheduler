use anyhow::Result;
use chrono::{DateTime, Utc};
use scheduler_core::{
    cache::Cache,
    error::SchedulerError,
    models::{Job, JobStatus},
    task::TaskManager,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct TaskFailureWatcher {
    task_manager: TaskManager,
    cache: Cache,
    check_interval: Duration,
    max_retries: i32,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl TaskFailureWatcher {
    pub fn new(
        task_manager: TaskManager,
        cache: Cache,
        check_interval: Duration,
        max_retries: i32,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Self {
        Self {
            task_manager,
            cache,
            check_interval,
            max_retries,
            initial_backoff,
            max_backoff,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting task failure watcher");
        loop {
            if let Err(e) = self.check_failed_jobs().await {
                error!("Error checking failed jobs: {}", e);
            }
            sleep(self.check_interval).await;
        }
    }

    async fn check_failed_jobs(&self) -> Result<()> {
        // Get all failed jobs from the database
        let failed_jobs = self
            .task_manager
            .get_jobs_by_status(JobStatus::Failed)
            .await?;

        for job in failed_jobs {
            if let Err(e) = self.handle_failed_job(job).await {
                error!("Error handling failed job {}: {}", job.id, e);
            }
        }

        Ok(())
    }

    async fn handle_failed_job(&self, job: Job) -> Result<()> {
        // Check if job has exceeded max retries
        if job.attempts >= job.max_attempts {
            info!(
                "Job {} has exceeded maximum retry attempts ({}), moving to dead letter queue",
                job.id, job.max_attempts
            );
            self.move_to_dead_letter_queue(job).await?;
            return Ok(());
        }

        // Calculate backoff based on current attempts
        let backoff = self.calculate_backoff(job.attempts as u32);
        info!(
            "Retrying job {} after {} seconds (attempt {}/{})",
            job.id,
            backoff.as_secs(),
            job.attempts + 1,
            job.max_attempts
        );

        // Wait for backoff period
        sleep(backoff).await;

        // Retry the job
        self.retry_job(job).await?;

        Ok(())
    }

    fn calculate_backoff(&self, retry_count: u32) -> Duration {
        let backoff = self.initial_backoff.as_secs() * 2u64.pow(retry_count);
        Duration::from_secs(backoff.min(self.max_backoff.as_secs()))
    }

    async fn retry_job(&self, mut job: Job) -> Result<()> {
        // Update job status to pending and increment attempts
        job.status = JobStatus::Pending;
        job.attempts += 1;

        // Update job in database
        self.task_manager
            .update_job_status(&job.id, job.status)
            .await?;
        self.task_manager.increment_job_attempts(&job.id).await?;

        // Push job back to queue
        if let Some(queue_name) = self.get_queue_name(&job) {
            self.cache
                .push_to_priority_queue(&queue_name, &job.id, job.priority)
                .await?;
        }

        Ok(())
    }

    async fn move_to_dead_letter_queue(&self, job: Job) -> Result<()> {
        // Update job status to indicate it's in dead letter queue
        self.task_manager
            .update_job_status(&job.id, JobStatus::Failed)
            .await?;

        // Store job in dead letter queue
        let dead_letter_queue = format!("dead_letter:{}", job.job_type.to_string());
        self.cache
            .push_to_queue(&dead_letter_queue, &job.id)
            .await?;

        info!("Moved job {} to dead letter queue", job.id);
        Ok(())
    }

    fn get_queue_name(&self, job: &Job) -> Option<String> {
        match job.job_type {
            JobType::OneTime => Some("one_time_jobs".to_string()),
            JobType::Recurring => Some("recurring_jobs".to_string()),
            JobType::Polling => Some("polling_jobs".to_string()),
        }
    }
}
