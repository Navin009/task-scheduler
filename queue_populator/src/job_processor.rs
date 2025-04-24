use scheduler_core::{cache::Cache, models::Job, task::TaskManager};
use sqlx::PgPool;
use tracing::{error, info};

use crate::error::{QueuePopulatorError, Result};

pub struct JobProcessor {
    db_pool: PgPool,
    cache: Cache,
    task_manager: TaskManager,
}

impl JobProcessor {
    pub async fn new(db_pool: PgPool, cache: Cache) -> Result<Self> {
        let task_manager = TaskManager::new(db_pool.clone());
        Ok(Self {
            db_pool,
            cache,
            task_manager,
        })
    }

    pub async fn process_jobs(&self) -> Result<()> {
        let due_jobs = self.fetch_due_jobs().await?;

        for job in due_jobs {
            if let Err(e) = self.push_job_to_queue(&job).await {
                error!("Failed to push job {} to queue: {}", job.id, e);
                continue;
            }

            if let Err(e) = self.update_job_status(&job).await {
                error!("Failed to update job {} status: {}", job.id, e);
            }
        }

        Ok(())
    }

    async fn fetch_due_jobs(&self) -> Result<Vec<Job>> {
        self.task_manager
            .get_due_jobs()
            .await
            .map_err(QueuePopulatorError::from)
    }

    async fn push_job_to_queue(&self, job: &Job) -> Result<()> {
        let job_json = serde_json::to_string(job)?;
        let queue_name = format!("jobs:{}", job.priority);

        self.cache
            .push_to_queue(&queue_name, &job_json)
            .await
            .map_err(QueuePopulatorError::from)
    }

    async fn update_job_status(&self, job: &Job) -> Result<()> {
        self.task_manager
            .update_job_status(job.id, "queued")
            .await
            .map_err(QueuePopulatorError::from)
    }
}
