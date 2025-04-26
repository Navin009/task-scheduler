use anyhow::Result;
use scheduler_core::{
    cache::Cache,
    task::{Job, JobStatus, JobType, TaskManager},
};
use std::time::Duration as StdDuration;
use tokio::time::sleep;
use tracing::{error, info};

pub struct TaskFailureWatcher {
    task_manager: TaskManager,
    cache: Cache,
    check_interval: StdDuration,
    max_retries: i32,
    initial_backoff: StdDuration,
    max_backoff: StdDuration,
}

impl TaskFailureWatcher {
    pub fn new(
        task_manager: TaskManager,
        cache: Cache,
        check_interval: StdDuration,
        max_retries: i32,
        initial_backoff: StdDuration,
        max_backoff: StdDuration,
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
                error!("Error handling failed job: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_failed_job(&self, job: Job) -> Result<()> {
        // If job has exceeded max retries, move to dead letter queue
        if job.retries >= job.max_retries {
            self.move_to_dead_letter_queue(job).await?;
        } else {
            // Otherwise, retry the job
            self.retry_job(job).await?;
        }

        Ok(())
    }

    fn calculate_backoff(&self, retry_count: u32) -> StdDuration {
        let backoff = self.initial_backoff.as_secs() * 2u64.pow(retry_count);
        StdDuration::from_secs(backoff.min(self.max_backoff.as_secs()))
    }

    async fn retry_job(&self, mut job: Job) -> Result<()> {
        // Calculate backoff delay
        let backoff = self.calculate_backoff(job.retries as u32);

        // Update job status to retrying
        self.task_manager
            .update_job_status(&job.id, JobStatus::Retrying)
            .await?;

        // Wait for backoff period
        sleep(backoff).await;

        // Update job status back to pending for retry
        self.task_manager
            .update_job_status(&job.id, JobStatus::Pending)
            .await?;

        // Increment attempts counter
        self.task_manager.increment_job_attempts(&job.id).await?;

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
