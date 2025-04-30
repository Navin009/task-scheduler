use anyhow::Result;
use chrono::{Duration, Utc};
use scheduler_core::{
    task::{Job, TaskManager},
    JobStatus,
};
use tracing::{error, info};

pub struct CleanupManager {
    task_manager: TaskManager,
    cleanup_interval: Duration,
    max_age: Duration,
}

impl CleanupManager {
    pub fn new(task_manager: TaskManager, cleanup_interval: Duration, max_age: Duration) -> Self {
        Self {
            task_manager,
            cleanup_interval,
            max_age,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting cleanup manager");
        loop {
            if let Err(e) = self.cleanup_old_jobs().await {
                error!("Error during cleanup: {}", e);
            }
            tokio::time::sleep(self.cleanup_interval.to_std().unwrap()).await;
        }
    }

    async fn cleanup(&self) -> Result<()> {
        self.cleanup_orphaned_jobs().await?;
        self.cleanup_old_jobs().await?;
        Ok(())
    }

    async fn cleanup_orphaned_jobs(&self) -> Result<()> {
        let cutoff_time = Utc::now() - Duration::hours(24); // Jobs older than 24 hours
        let orphaned_jobs = self
            .task_manager
            .get_jobs_by_status_and_time(JobStatus::Running, cutoff_time)
            .await?;

        for job in orphaned_jobs {
            info!("Cleaning up orphaned job: {}", job.id);
            self.mark_job_as_failed(job).await?;
        }

        Ok(())
    }

    async fn cleanup_old_jobs(&self) -> Result<()> {
        let cutoff_time = Utc::now() - self.max_age;
        let jobs = self.task_manager.get_jobs_older_than(cutoff_time).await?;

        for job in jobs {
            if let Err(e) = self.archive_job(job).await {
                error!("Error archiving job: {}", e);
            }
        }

        Ok(())
    }

    async fn mark_job_as_failed(&self, job: Job) -> Result<()> {
        // Update job status to failed
        self.task_manager
            .update_job_status(&job.id, JobStatus::Failed)
            .await?;

        // If job has exceeded max retries, move to dead letter queue
        if job.retries >= job.max_retries {
            let dead_letter_queue = format!("dead_letter:{}", job.job_type.to_string());
            self.task_manager
                .move_to_dead_letter_queue(&job.id, &dead_letter_queue)
                .await?;
        }

        Ok(())
    }

    async fn archive_job(&self, job: Job) -> Result<()> {
        // Move job to archive table
        self.task_manager.archive_job(&job.id).await?;
        info!("Archived job: {}", job.id);
        Ok(())
    }
}
